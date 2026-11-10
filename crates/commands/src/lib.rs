//! The one binary's terminal side: a subcommand runs here and never starts
//! egui or wgpu. The arguments live in `toolbox-cli`, the work in the same
//! core, system and apply code the window uses.

pub mod accounts;
mod act;
pub mod apply;
pub mod boot;
pub mod bootrepair;
pub mod change;
pub mod configs;
pub mod diagnostics;
pub mod doctor;
pub mod filter;
mod guide;
mod help;
mod interrupt;
pub mod lists;
pub mod locales;
pub mod names;
pub mod network;
mod out;
pub mod packages;
mod pager;
#[cfg(test)]
mod parity;
mod queue;
mod reads;
mod root;
pub mod settings;
pub mod setup;
mod spinner;
mod start;
pub mod status;
mod terminal;
pub mod window;
mod written;

use std::io::Write;
use std::sync::Arc;

use serde::Serialize;
use tokio::runtime::Runtime;
use toolbox_catalog::ToggleCatalog;
use toolbox_cli::Exit;
use toolbox_cli::args::{
    AppsCommand, AppsList, Command, Diagnostics, KernelsCommand, KernelsList, LocalesList,
    MaintenanceCommand, Read, ReadList, ServicesList, ToggleCommand, ToggleList, TogglePage,
    Update,
};
use toolbox_cli::output::{ColorMode, Row, json};
use toolbox_core::features::maintenance::RunNow;
use toolbox_core::{Snapshot, Toggle};
use toolbox_plan::protocol::Mode;
use toolbox_plan::{Plan, UnitScope};
use toolbox_privatebin::{Https, Transport};
use toolbox_services::apply::Launcher as _;
use toolbox_services::catalogs::{Catalogs, catalog_dir};
use toolbox_services::diagnostics::DryRunTransport;
use toolbox_services::page::PageId;
use toolbox_services::programs::Programs;
use toolbox_services::units::{Systemd, UnitLister as _};
use toolbox_services::{desktop, i18n};
use toolbox_system::{Sources, SystemDiagnostics, probe_snapshot};
use written::Written;

use self::change::KernelChange;
use self::lists::listing;
use self::out::{Json, Listing, Out, Settings, print, unavailable};
pub use self::out::{error_text, say_error};
use self::pager::Pager;
pub use self::start::{parse_failed, usage};
use self::terminal::Terminal;

#[derive(Serialize)]
struct PageRow {
    id: &'static str,
    name: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub verbose: bool,
    pub no_pager: bool,
    pub color: ColorMode,
    pub width: Option<usize>,
    pub quiet: bool,
}

pub fn main(command: Command, options: Options) -> Exit {
    let settings = Settings {
        color: options.color,
        width: options.width,
        quiet: options.quiet,
        verbose: options.verbose,
        json: Json::Off,
    };

    match Runtime::new() {
        Ok(runtime) => run(command, &runtime, options.no_pager, settings),
        Err(error) => {
            tracing::error!("no runtime for the command: {error}");
            Exit::Failure
        }
    }
}

fn run(command: Command, runtime: &Runtime, no_pager: bool, mut settings: Settings) -> Exit {
    if let Some(confirm) = command.confirm()
        && confirm.json
    {
        // Checked before anything asks, a password included.
        if confirm.cannot_ask() {
            return apply::cannot_ask(settings);
        }

        settings.json = Json::Lines;
    } else if reads::json_read(&command) {
        settings.json = Json::Read;
    } else {
        settings.json = Json::Off;
    }

    if let Some((pager, input)) = Pager::open(&command, no_pager, runtime) {
        let mut written = Written::new(std::io::BufWriter::new(input));
        let (exit, kept) = written_out(command, runtime, &mut written, settings);
        let exit = written::ended(written.finish(exit), kept, settings);
        pager.wait();
        return exit;
    }

    let mut written = Written::new(std::io::stdout().lock());
    let (exit, kept) = written_out(command, runtime, &mut written, settings);
    written::ended(written.finish(exit), kept, settings)
}

fn written_out(
    command: Command,
    runtime: &Runtime,
    writer: &mut dyn Write,
    settings: Settings,
) -> (Exit, Vec<out::Said>) {
    let mut out = Out::new(writer, settings);
    let exit = dispatch(command, runtime, &mut out);
    (exit, out.into_kept())
}

fn dispatch(command: Command, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    match command {
        Command::Status(args) => status::status(args, runtime.handle(), out),
        Command::Doctor(read) => doctor::run(read, runtime, out),
        Command::Pages(read) => pages(read, out),
        Command::Guide(command) => guide::command(command, out),
        Command::Diagnostics(args) => diagnose(&args, runtime, out),
        Command::Toggle(ToggleCommand::List(list)) => toggle(&list, out),
        Command::Apps(AppsCommand::List(list)) => apps(&list, out),
        Command::Update(args) => update(&args, runtime, out),
        Command::Services(command) => match command.change() {
            Ok((change, unit)) => act::service(change, unit, runtime, out),
            Err(list) => services(list, runtime, out),
        },
        Command::Toggle(ToggleCommand::On(args)) => act::toggle(&args, Toggle::On, runtime, out),
        Command::Toggle(ToggleCommand::Off(args)) => act::toggle(&args, Toggle::Off, runtime, out),
        Command::Apps(AppsCommand::Install(args)) => act::app(&args, Toggle::On, runtime, out),
        Command::Apps(AppsCommand::Remove(args)) => act::app(&args, Toggle::Off, runtime, out),
        Command::Kernels(KernelsCommand::List(list)) => kernels(&list, out),
        Command::Kernels(KernelsCommand::Install(args)) => {
            act::kernel(&args, KernelChange::Install, runtime, out)
        }
        Command::Kernels(KernelsCommand::Remove(args)) => {
            act::kernel(&args, KernelChange::Remove, runtime, out)
        }
        Command::Kernels(KernelsCommand::Headers(args)) => {
            act::kernel(&args, KernelChange::Headers, runtime, out)
        }
        Command::Maintenance(MaintenanceCommand::List(read)) => act::tasks(&read, out),
        Command::Maintenance(MaintenanceCommand::Run(args)) => act::task(&args, runtime, out),
        Command::Kernels(KernelsCommand::Default(args)) => {
            boot::kernel_default(&args, runtime, out)
        }
        Command::Dkms(dkms) => boot::dkms(dkms, runtime, out),
        Command::Locales(command) => locales::command(command, runtime, out),
        Command::Users(command) => accounts::users_command(command, runtime, out),
        Command::Groups(command) => accounts::groups_command(command, runtime, out),
        Command::Boot(command) => boot::command(command, runtime, out),
        Command::Configs(command) => configs::command(command, runtime, out),
        Command::Network(command) => network::command(command, runtime, out),
        Command::Packages(command) => packages::command(command, runtime, out),
        Command::BootRepair(command) => bootrepair::command(command, runtime, out),
        Command::Setup(command) => setup::command(command, runtime, out),
        Command::Settings(command) => settings::command(command, out),
        Command::Queue(command) => queue::command(command, runtime, out),
        Command::Completions(args) => help::completions(&args, out),
        Command::Help(args) => help::help(&args, out),
    }
}

fn services(list: &ServicesList, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let scope = UnitScope::from_user(list.user);

    let listed = spinner::while_reading(list.read.json || out.quiet(), || {
        runtime.block_on(Systemd.list(scope))
    });

    match listed {
        Ok(units) => filter::print(
            &list.read,
            &lists::services(&units),
            &filter::services(list),
            out,
            |rows| listing::services(rows, list.user),
        ),
        Err(error) => {
            tracing::warn!("the units could not be listed: {error}");
            unavailable("the services could not be read", out)
        }
    }
}

fn toggle(list: &ToggleList, out: &mut Out<'_>) -> Exit {
    let catalogs = catalogs();
    let pages: Vec<(TogglePage, &ToggleCatalog)> = [
        (TogglePage::Components, &catalogs.components),
        (TogglePage::Tweaks, &catalogs.settings),
        (TogglePage::Gaming, &catalogs.gaming),
        (TogglePage::Setup, &catalogs.setup),
    ]
    .into_iter()
    .filter(|(page, _)| list.page.is_none_or(|wanted| wanted == *page))
    .filter_map(|(page, toggles)| toggles.catalog.as_ref().ok().map(|found| (page, found)))
    .collect();

    let current = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let rows = lists::toggles(&pages, &snapshot(&catalogs), &desktop::desktops(&current));
    filter::print(&list.read, &rows, &filter::toggles(list), out, |rows| {
        listing::toggles(rows, "Switches", "garuda-toolbox toggle on <id>")
    })
}

fn apps(list: &AppsList, out: &mut Out<'_>) -> Exit {
    let catalogs = catalogs();
    let rows = lists::apps(
        &catalogs.apps.apps,
        list.category.as_ref().map(AsRef::as_ref),
        &snapshot(&catalogs),
    );

    filter::print(&list.read, &rows, &filter::apps(list), out, |rows| {
        listing::toggles(rows, "Apps", "garuda-toolbox apps install <id>")
    })
}

fn kernels(list: &KernelsList, out: &mut Out<'_>) -> Exit {
    match spinner::while_reading(list.read.json || out.quiet(), || lists::kernels(&system())) {
        Some(rows) => filter::print(
            &list.read,
            &rows,
            &filter::kernels(list),
            out,
            listing::kernels,
        ),
        None => unavailable("pacman's databases could not be read", out),
    }
}

fn locales(list: &LocalesList, out: &mut Out<'_>) -> Exit {
    let rows = spinner::while_reading(list.read.json || out.quiet(), || lists::locales(&system()));
    filter::print(
        &list.read,
        &rows,
        &filter::locales(list),
        out,
        listing::locales,
    )
}

/// The people's accounts and root, every account with `all`. `--json`
/// leaves out the same ones.
fn users(list: &ReadList, all: bool, snapshot: &Snapshot, out: &mut Out<'_>) -> Exit {
    match lists::users(snapshot) {
        Some(rows) => {
            let (shown, hidden) = lists::people(rows, all);
            filter::print_all(list, &shown, out, |people| listing::users(people, hidden))
        }
        None => unavailable("the accounts could not be read", out),
    }
}

fn groups(list: &ReadList, snapshot: &Snapshot, out: &mut Out<'_>) -> Exit {
    match lists::groups(snapshot) {
        Some(rows) => filter::print_all(list, &rows, out, listing::groups),
        None => unavailable("the accounts could not be read", out),
    }
}

fn pages(read: Read, out: &mut Out<'_>) -> Exit {
    let rows: Vec<PageRow> = PageId::ALL
        .iter()
        .map(|page| PageRow {
            id: page.slug(),
            // English, as all command line output, whatever the session.
            name: i18n::text_in(i18n::FALLBACK, page.text_id()).into_owned(),
        })
        .collect();

    print(read, &rows, out, |rows| Listing {
        title: "Pages",
        summary: Some(rows.len().to_string()),
        head: &["PAGE", "NAME"],
        rows: rows
            .iter()
            .map(|row| Row::plain(vec![row.id.to_owned(), row.name.clone()]))
            .collect(),
        hint: Some("garuda-toolbox --page <page>".to_owned()),
    })
}

fn diagnose(args: &Diagnostics, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    if args.confirm.json {
        say_error(
            out,
            "diagnostics prints its report as text, leave out --json",
        );
        return Exit::Usage;
    }

    let launcher = Arc::new(root::helper(out.settings().color));
    let transport: Arc<dyn Transport> = if launcher.mode() == Mode::DryRun {
        Arc::new(DryRunTransport)
    } else {
        Arc::new(Https::new())
    };

    diagnostics::collect(
        args.report.0,
        // Diagnostics change nothing, so a dry run only keeps the
        // report from going out.
        args.upload && !args.confirm.dry_run,
        args.confirm.yes,
        runtime.handle(),
        Arc::new(SystemDiagnostics::system(launcher)),
        transport,
        &mut Terminal {
            color: out.settings().color,
        },
        out,
    )
}

fn update(args: &Update, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let snapshot = system();
    let garuda_update = Programs::check().garuda_update;

    let mut plans = vec![RunNow::UpgradeSystem.plan(&snapshot, garuda_update)];
    if args.aur {
        plans.push(RunNow::AurUpgrade.plan(&snapshot, garuda_update));
    }

    let plan = Plan::new(
        plans
            .iter()
            .flat_map(|plan| plan.steps().iter().map(|step| step.action.clone())),
    );

    apply::carry_out(
        plan,
        args.confirm,
        None,
        runtime.handle(),
        root::launcher(out.settings().color),
        &mut Terminal {
            color: out.settings().color,
        },
        out,
    )
}

fn catalogs() -> Catalogs {
    Catalogs::load(&catalog_dir(|key| std::env::var_os(key)))
}

fn snapshot(catalogs: &Catalogs) -> Snapshot {
    let sources = Sources::system().with_config_edits(catalogs.config_edits());

    probe_snapshot(&sources).0
}

/// The system without the toggle catalogs.
fn system() -> Snapshot {
    probe_snapshot(&Sources::system()).0
}
