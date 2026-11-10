//! `garuda-toolbox`: maintain and configure an Arch based system.

mod ports;
mod start;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use clap::FromArgMatches as _;
use eframe::CreationContext;
use tokio::runtime::Handle;
use toolbox_cli::Cli;
use toolbox_commands::window::{Opening, opening};
use toolbox_config::{Paths, Settings};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui::catalogs::{Catalogs, catalog_dir};
use toolbox_gui::desktop::files::DesktopFiles;
use toolbox_gui::desktop::{self, follow};
use toolbox_gui::instance::{Single, claim};
use toolbox_gui::notifications::DesktopNotifier;
use toolbox_gui::page::PageId;
use toolbox_gui::pages::kernels::view::{self as kernel_view, dkms_answer};
use toolbox_gui::programs::Programs;
use toolbox_gui::queue::Restored;
use toolbox_gui::reader::ReaderHandle;
use toolbox_gui::units::{self, Systemd, UnitsHandle};
use toolbox_gui::{App, Setup, i18n, log_folder, panics, queue, reader, snapshot};
use toolbox_gui_kit::desktop::family::Family;
use toolbox_log::subscriber;
use toolbox_system::{
    Dkms, DkmsSource as _, FactSources, Facts, Sources, Status, StatusSources, probe_facts,
    probe_snapshot, probe_status,
};
use tracing::level_filters::LevelFilter;

use self::ports::{
    helper_services, kept_queue, machine, maintenance, network, news_reader, packages, pictures,
    repair,
};
use self::start::{load_settings, preferences};

/// The Wayland `app_id`, the desktop file's name and eframe's storage folder.
const APP_ID: &str = "garuda-toolbox";
/// The log file is `<log folder>/<PROGRAM>.log`.
const PROGRAM: &str = "garuda-toolbox";

struct Start {
    handle: Handle,
    family: Family,
    /// The queue restored from `changes.json`, written there again once
    /// the window can tell of a failed write.
    changes: ChangeSet,
    restored: Restored,
    /// How many saved changes did not come back, for Home's banner.
    lost: usize,
    changes_file: PathBuf,
    catalogs: Catalogs,
    desktops: Vec<String>,
    settings: Settings,
    paths: Paths,
    /// `--boot-repair`: Boot Repair shows on an installed system too.
    boot_repair: bool,
    /// The command bus and where later starts arrive, made before the
    /// window so a start that comes meanwhile is kept.
    instance: Single,
    /// The page a start option opens: `--setup-assistant`.
    open: Option<PageId>,
}

struct Readers {
    dkms: ReaderHandle<kernel_view::Dkms>,
    status: ReaderHandle<Status>,
    facts: ReaderHandle<Facts>,
    programs: ReaderHandle<Programs>,
    units: UnitsHandle,
}

type Created = Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> ExitCode {
    // A shell hook asking for completions gets them and nothing else: no
    // log, no root check, no window.
    toolbox_cli::complete::run(toolbox_commands::names::of);

    let parsed = toolbox_cli::command()
        .try_get_matches()
        .and_then(|matches| Cli::from_arg_matches(&matches));

    let cli = match parsed {
        Ok(cli) => cli,
        Err(error) => return toolbox_commands::parse_failed(&error).into(),
    };

    if rustix::process::geteuid().is_root() {
        return refuse(
            "do not run it as root\nrun it as yourself, it asks for root rights when a change needs them",
        );
    }

    // Before anything logs: the subscriber is also the `log` logger, which
    // only the first to ask becomes. A command logs as fully as the window,
    // but to the file alone: its own output says what the user must see.
    subscriber::init(
        LevelFilter::INFO,
        std::env::var_os(subscriber::ENV).as_deref(),
        profiler(),
    );
    if cli.command.is_some() {
        toolbox_log::quiet_terminal();
    }

    let paths = Paths::from_process();
    panics::log_panics(paths.crash_file());
    if cli.command.is_some() {
        // The window reads its settings later, a command has no later.
        load_settings(&paths);
    }

    toolbox_log::open_file(log_folder::log_dir(&paths).as_deref(), PROGRAM);

    if let Some(command) = cli.command {
        let options = toolbox_commands::Options {
            verbose: cli.verbose,
            no_pager: cli.no_pager,
            color: cli.color,
            width: cli.width,
            quiet: cli.quiet,
        };

        return toolbox_commands::main(command, options).into();
    }

    let opening = match opening(&cli.window) {
        Ok(opening) => opening,
        Err(error) => return toolbox_commands::usage(&error.to_string()).into(),
    };

    match run(&paths, opening, cli.window.boot_repair) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => refuse(&error.to_string()),
    }
}

fn run(
    paths: &Paths,
    opening: Opening,
    boot_repair: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("toolbox-worker")
        .enable_all()
        .build()?;

    let Some((instance, held)) = claim(&runtime, opening) else {
        return Ok(());
    };

    let catalog = catalog_dir(|key| std::env::var_os(key));

    if !catalog.is_dir() {
        tracing::error!("the catalog folder {} is missing", catalog.display());
        return Ok(toolbox_gui::missing::run(APP_ID, catalog)?);
    }

    let handle = runtime.handle().clone();
    let current_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    // Restored before the system is known. The queue meets the first
    // snapshot once the service read it.
    let changes_file = paths.staged_changes_file();
    let (changes, problems) = queue::read(&changes_file, &Snapshot::default());
    let settings = load_settings(paths);

    let start = Start {
        family: Family::from_current_desktop(&current_desktop),
        changes,
        restored: Restored::of(&problems),
        lost: problems.len(),
        changes_file,
        catalogs: Catalogs::load(&catalog),
        desktops: toolbox_services::desktop::desktops(&current_desktop),
        handle,
        settings,
        paths: paths.clone(),
        boot_repair,
        open: opening.page,
        instance,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_inner_size([1100.0, 760.0])
            .with_min_inner_size([640.0, 480.0]),
        renderer: eframe::Renderer::Wgpu,
        persist_window: true,
        ..Default::default()
    };

    let ran = eframe::run_native(
        "Garuda Toolbox",
        options,
        Box::new(move |cc| create(cc, start)),
    );

    // The window wrote the queue on close, a write the task had begun
    // still gets its moment. The bus name goes first, while its runtime runs.
    drop(held);
    runtime.shutdown_timeout(Duration::from_secs(2));
    ran?;
    Ok(())
}

/// The snapshot's reader. Its sources include the files the catalogs edit.
fn system_snapshot(
    handle: &Handle,
    ctx: &egui::Context,
    catalogs: &Catalogs,
) -> snapshot::SnapshotHandle {
    let sources = Sources::system().with_config_edits(catalogs.config_edits());

    let wake = toolbox_gui::waking::repaint(ctx.clone());
    snapshot::spawn(handle, wake, move || probe_snapshot(&sources))
}

/// Starts the services that need the window's context, then the app.
#[cfg_attr(
    not(feature = "selftest"),
    expect(clippy::unnecessary_wraps, reason = "only the self-test can fail here")
)]
fn create(cc: &CreationContext<'_>, start: Start) -> Created {
    let Start {
        handle,
        family,
        changes,
        restored,
        lost,
        changes_file,
        catalogs,
        desktops,
        settings,
        paths,
        boot_repair,
        open,
        instance,
    } = start;

    let helper = helper_services(&handle, &cc.egui_ctx);

    let readers = readers(&handle, &cc.egui_ctx, &settings);
    let commands = instance.commands.sender.clone();
    let (power, starter) = machine(&handle, helper.dry_run, commands, &cc.egui_ctx);
    let setup = Setup {
        look: Some(look(&handle, &cc.egui_ctx, family, &catalogs)),
        system_language: i18n::language_from_env(|key| std::env::var_os(key)),
        queue: kept_queue((changes, restored), changes_file, &handle, &cc.egui_ctx),
        queue_lost: lost,
        apply: Some(helper.apply),
        notifier: Some(Arc::new(DesktopNotifier::new(handle.clone()))),
        snapshot: Some(system_snapshot(&handle, &cc.egui_ctx, &catalogs)),
        dkms: Some(readers.dkms),
        status: Some(readers.status),
        facts: Some(readers.facts),
        news: Some(news_reader(&handle, &cc.egui_ctx, helper.dry_run)),
        programs: Some(readers.programs),
        units: Some(readers.units),
        pictures: Some(pictures(&cc.egui_ctx)),
        power: Some(power),
        launcher: Some(starter),
        maintenance: Some(maintenance(&handle, helper.dry_run)),
        network: Some(network(&handle, helper.dry_run)),
        packages: Some(packages(&handle, helper.dry_run)),
        users: Some(helper.users),
        configs: Some(helper.configs),
        boot_repair: Some(repair(&handle, helper.dry_run, boot_repair)),
        open,
        commands: Some(instance.commands),
        arrivals: instance.arrivals,
        diagnostics: Some(helper.diagnostics),
        output_share: Some(helper.output_share),
        boot: Some(helper.boot),
        catalogs,
        desktops,
        version: env!("CARGO_PKG_VERSION"),
        dry_run: helper.dry_run,
        preferences: preferences(&paths, settings, &cc.egui_ctx),
    };

    #[cfg(feature = "selftest")]
    if let Some((dir, suite)) = toolbox_gui::selftest::requested(|key| std::env::var_os(key)) {
        toolbox_gui::selftest::install(&cc.egui_ctx, dir, &suite)?;
    }

    Ok(Box::new(App::new(cc, setup)))
}

/// The slow reads besides the snapshot. The status checks again on the
/// period the settings name.
fn readers(handle: &Handle, ctx: &egui::Context, settings: &Settings) -> Readers {
    let repaint = || toolbox_gui::waking::repaint(ctx.clone());
    let program = Dkms::system();
    let status = reader::spawn(handle, repaint(), "the status", || {
        probe_status(&StatusSources::system())
    });

    status.every(settings.status.refresh.period());

    // One set of sources, so libalpm unpacks the sync databases once.
    let fact_sources = FactSources::system();
    let facts = reader::spawn(handle, repaint(), "the facts", move || {
        probe_facts(&fact_sources)
    });

    let units = units::spawn(handle, repaint(), Arc::new(Systemd), units::SETTLE);

    units::watch::spawn(handle, &units.requests());

    Readers {
        dkms: reader::spawn_fallible(handle, repaint(), "dkms", move || {
            dkms_answer(program.status())
        }),
        status,
        facts,
        programs: reader::spawn(handle, repaint(), "the programs", Programs::check),
        units,
    }
}

/// The settings, with the log detail they ask for. A broken file still
/// gives the defaults, and the log says what was wrong.
/// Tracy's layer in a profiling build.
#[cfg(feature = "profiling")]
#[expect(
    clippy::unnecessary_wraps,
    reason = "a build without the feature has no layer"
)]
fn profiler() -> Option<subscriber::Profiler> {
    Some(toolbox_gui::profiling::layer())
}

#[cfg(not(feature = "profiling"))]
const fn profiler() -> Option<subscriber::Profiler> {
    None
}

fn refuse(message: &str) -> ExitCode {
    tracing::error!("{message}");

    // The log may not exist yet, the terminal is the other place to look.
    write!(
        std::io::stderr(),
        "{}",
        toolbox_commands::error_text(message)
    )
    .ok();
    ExitCode::FAILURE
}

fn look(
    handle: &Handle,
    ctx: &egui::Context,
    family: Family,
    catalogs: &Catalogs,
) -> tokio::sync::watch::Receiver<desktop::DesktopLook> {
    follow::spawn(
        handle,
        ctx.clone(),
        family,
        DesktopFiles::from_process(),
        catalogs.app_icon_names().into(),
    )
}
