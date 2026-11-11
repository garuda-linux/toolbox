mod aliases;
mod confirm;
mod defaults;
mod global;
mod queue;
mod show;
mod units;

use toolbox_plan::PageName;
use toolbox_state::diagnostics::{Collector, Selection};

use super::pages::Report;
use super::{Cli, Command, Group, ServicesCommand, ToggleCommand, TogglePage};

pub(super) fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_from_words(std::iter::once("garuda-toolbox").chain(args.iter().copied()))
}

#[test]
fn no_arguments_open_the_window() {
    let cli = parse(&[]).unwrap();

    assert!(cli.command.is_none());
    assert_eq!(cli.window.page, None);
}

#[test]
fn the_desktop_file_actions_keep_working() {
    let cli = parse(&["--page", "maintenance"]).unwrap();
    assert_eq!(cli.window.page, Some(PageName::new("maintenance").unwrap()));

    assert!(
        parse(&["--setup-assistant"])
            .unwrap()
            .window
            .setup_assistant
    );
    assert!(parse(&["--boot-repair"]).unwrap().window.boot_repair);
}

#[test]
fn an_action_needs_its_page() {
    parse(&["--action", "inxi"]).unwrap_err();

    let cli = parse(&["--page", "diagnostics", "--action", "inxi"]).unwrap();
    assert_eq!(cli.window.action, Some(PageName::new("inxi").unwrap()));
}

#[test]
fn a_page_name_is_checked_before_anything_runs() {
    parse(&["--page", "Maintenance"]).unwrap_err();
    parse(&["--page", "../etc"]).unwrap_err();

    let error = parse(&["--page", "diagnostics", "--action", &"a".repeat(33)]).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("small letters and hyphens, at most 32"),
        "{error}"
    );
}

#[test]
fn window_options_and_a_subcommand_do_not_mix() {
    parse(&["--page", "home", "status"]).unwrap_err();
}

#[test]
fn read_commands_take_json() {
    let status = parse(&["status", "--json"]).unwrap();
    assert!(matches!(status.command, Some(Command::Status(status)) if status.read.json));

    let services = parse(&["services", "list", "--user", "--json"]).unwrap();
    assert!(matches!(
        services.command,
        Some(Command::Services(ServicesCommand::List(list))) if list.user && list.read.json
    ));
}

#[test]
fn toggles_list_one_page_or_all() {
    let cli = parse(&["toggle", "list", "--page", "tweaks"]).unwrap();
    let Some(Command::Toggle(ToggleCommand::List(list))) = cli.command else {
        panic!("not a toggle list");
    };

    assert_eq!(list.page, Some(TogglePage::Tweaks));
    parse(&["toggle", "list", "--page", "kernels"]).unwrap_err();
}

#[test]
fn diagnostics_names_a_report_of_the_page() {
    let cli = parse(&["diagnostics", "inxi"]).unwrap();
    let Some(Command::Diagnostics(diagnostics)) = cli.command else {
        panic!("not diagnostics");
    };

    assert_eq!(diagnostics.report, Report(Selection::One(Collector::Inxi)));
    assert!(!diagnostics.upload);
    parse(&["diagnostics", "everything"]).unwrap_err();
}

#[test]
fn a_change_may_skip_its_question() {
    let cli = parse(&["update", "--aur", "--yes"]).unwrap();
    let Some(Command::Update(update)) = cli.command else {
        panic!("not update");
    };

    assert!(update.aur);
    assert!(update.confirm.yes);
}

#[test]
fn every_command_has_help_with_an_example() {
    let mut command = <Cli as clap::CommandFactory>::command();
    command.build();

    let mut pending: Vec<(String, &clap::Command)> = command
        .get_subcommands()
        .map(|sub| (sub.get_name().to_owned(), sub))
        .collect();

    let mut seen = 0;

    while let Some((path, sub)) = pending.pop() {
        let after = sub
            .get_after_help()
            .map(ToString::to_string)
            .unwrap_or_default();

        assert!(after.contains("garuda-toolbox"), "`{path}` has no example");

        seen += 1;
        pending.extend(
            sub.get_subcommands()
                .map(|next| (format!("{path} {}", next.get_name()), next)),
        );
    }

    assert!(
        seen > command.get_subcommands().count(),
        "only the top level was walked"
    );
}

#[test]
fn the_definition_is_consistent() {
    <Cli as clap::CommandFactory>::command().debug_assert();
}

#[test]
fn the_help_leaves_the_exit_codes_out() {
    let help = crate::command().render_long_help().to_string();

    assert!(!help.contains("Exit codes"), "the help lists exit codes");
}

#[test]
fn every_example_heading_takes_the_help_header_style() {
    let mut command = crate::command();
    command.build();
    let styles = crate::output::Paint::help();
    let header = styles.get_header().render().to_string();

    let mut helps = vec![command.render_help().ansi().to_string()];
    for sub in command.get_subcommands_mut() {
        helps.push(sub.render_help().ansi().to_string());
    }

    for help in helps {
        if help.contains("Example") {
            assert!(
                help.contains(&format!("{header}Example")),
                "an example heading is plain:\n{help}"
            );
        }
    }
}

#[test]
fn every_service_change_names_its_unit() {
    for (word, change) in [
        ("start", super::UnitChange::Start),
        ("stop", super::UnitChange::Stop),
        ("restart", super::UnitChange::Restart),
        ("reload", super::UnitChange::Reload),
        ("enable", super::UnitChange::Enable),
        ("disable", super::UnitChange::Disable),
        ("mask", super::UnitChange::Mask),
        ("unmask", super::UnitChange::Unmask),
    ] {
        let cli = parse(&["services", word, "sshd.service", "--user", "--yes"]).unwrap();
        let Some(Command::Services(services)) = cli.command else {
            panic!("{word} is no services command");
        };

        let (found, unit) = services.change().unwrap();
        assert_eq!(found, change);
        assert_eq!(unit.unit.as_str(), "sshd.service");
        assert!(unit.user && unit.confirm.yes);
    }
}

#[test]
fn a_name_with_a_shell_character_is_refused() {
    parse(&["services", "start", "a;b.service"]).unwrap_err();
    parse(&["toggle", "on", "$(id)"]).unwrap_err();
    parse(&["apps", "install", "fire fox"]).unwrap_err();
}

#[test]
fn switches_apps_kernels_and_tasks_take_their_id() {
    assert!(matches!(
        parse(&["toggle", "on", "pipewire"]).unwrap().command,
        Some(Command::Toggle(ToggleCommand::On(_)))
    ));
    assert!(matches!(
        parse(&["apps", "remove", "firefox", "-y"]).unwrap().command,
        Some(Command::Apps(super::AppsCommand::Remove(_)))
    ));
    assert!(matches!(
        parse(&["kernels", "headers", "linux"]).unwrap().command,
        Some(Command::Kernels(super::KernelsCommand::Headers(_)))
    ));
    assert!(matches!(
        parse(&["maintenance", "run", "remote-full-fix"])
            .unwrap()
            .command,
        Some(Command::Maintenance(super::MaintenanceCommand::Run(_)))
    ));
    assert!(matches!(
        parse(&["toggle", "list", "--page", "setup"])
            .unwrap()
            .command,
        Some(Command::Toggle(ToggleCommand::List(_)))
    ));
}

#[test]
fn the_tweaks_page_is_named_as_the_window_names_it() {
    for name in ["system-settings", "tweaks"] {
        let cli = parse(&["toggle", "list", "--page", name]).unwrap();
        let Some(Command::Toggle(ToggleCommand::List(list))) = cli.command else {
            panic!("not a toggle list");
        };

        assert_eq!(list.page, Some(TogglePage::Tweaks));
    }
}

#[test]
fn every_domain_name_is_checked_as_its_value_object() {
    for argv in [
        &["users", "lock", "Bad Name"][..],
        &["groups", "add", "a;b"],
        &["users", "add", "ada", "--shell", "bash"],
        &["users", "edit", "ada", "--home", "relative/path"],
        &["locales", "enable", "de_DE UTF-8"],
        &["locales", "packs", "install", "fire fox"],
        &["kernels", "install", "linux;rm"],
        &["boot", "param", "add", "a b"],
        &["network", "drivers", "allow", "e1000;x"],
        &["network", "ping", "-bad"],
    ] {
        assert!(parse(argv).is_err(), "{argv:?} was taken");
    }
}

#[test]
fn a_good_domain_name_arrives_as_its_value_object() {
    let Some(Command::Kernels(super::KernelsCommand::Install(args))) =
        parse(&["kernels", "install", "linux-lts"]).unwrap().command
    else {
        panic!("no kernels install");
    };

    assert_eq!(
        args.kernel,
        toolbox_plan::PackageName::new("linux-lts").unwrap()
    );
}

#[test]
fn verbose_follows_the_command() {
    assert!(parse(&["status", "--verbose"]).unwrap().verbose);
    assert!(parse(&["services", "list", "-v"]).unwrap().verbose);
    assert!(!parse(&["status"]).unwrap().verbose);
}

#[test]
fn read_verbs_are_list_and_show_with_the_old_names_as_aliases() {
    use super::{BootRepairCommand, DkmsCommand, NetworkCommand};

    for verb in ["list", "status"] {
        let cli = parse(&["dkms", verb, "--json"]).unwrap();
        assert!(matches!(cli.command, Some(Command::Dkms(DkmsCommand::List(read))) if read.json));
    }

    for verb in ["show", "status"] {
        let cli = parse(&["network", verb]).unwrap();
        assert!(matches!(
            cli.command,
            Some(Command::Network(Group(NetworkCommand::Show(_))))
        ));

        let repair = parse(&["bootrepair", verb]).unwrap();
        assert!(matches!(
            repair.command,
            Some(Command::BootRepair(BootRepairCommand::Show(_)))
        ));
    }
}

#[test]
fn settings_get_takes_json() {
    use super::{SettingKey, SettingsCommand};

    let cli = parse(&["settings", "get", "language", "--json"]).unwrap();
    let Some(Command::Settings(Group(SettingsCommand::Get(get)))) = cli.command else {
        panic!("not settings get");
    };

    assert_eq!(get.key, SettingKey::Language);
    assert!(get.read.json);
}

#[test]
fn users_list_shows_every_account_only_with_all() {
    use super::UsersCommand;

    for (argv, all) in [
        (&["users", "list", "--json"][..], false),
        (&["users", "list", "--all"], true),
    ] {
        let Some(Command::Users(UsersCommand::List(list))) = parse(argv).unwrap().command else {
            panic!("not users list");
        };

        assert_eq!(list.all, all, "{argv:?}");
        assert_eq!(list.read.json, !all, "{argv:?}");
    }
}

#[test]
fn a_dry_run_shows_the_plan_and_never_goes_with_yes() {
    use super::KernelsCommand;

    let Some(Command::Update(update)) = parse(&["update", "--dry-run"]).unwrap().command else {
        panic!("not update");
    };

    assert!(update.confirm.dry_run);
    assert!(!update.confirm.yes);

    let argv = ["kernels", "install", "linux-lts", "--dry-run"];
    let Some(Command::Kernels(KernelsCommand::Install(install))) = parse(&argv).unwrap().command
    else {
        panic!("not kernels install");
    };

    assert!(install.confirm.dry_run);

    let error = parse(&["update", "--dry-run", "--yes"]).unwrap_err();
    assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
}

#[test]
fn status_takes_brief_next_to_json() {
    let brief = parse(&["status", "--brief"]).unwrap();
    assert!(
        matches!(brief.command, Some(Command::Status(status)) if status.brief && !status.read.json)
    );

    let both = parse(&["status", "--brief", "--json"]).unwrap();
    assert!(
        matches!(both.command, Some(Command::Status(status)) if status.brief && status.read.json)
    );
}

#[test]
fn doctor_reads_and_takes_json() {
    let text = parse(&["doctor"]).unwrap();
    assert!(matches!(text.command, Some(Command::Doctor(read)) if !read.json));

    let json = parse(&["doctor", "--json"]).unwrap();
    assert!(matches!(json.command, Some(Command::Doctor(read)) if read.json));
}
