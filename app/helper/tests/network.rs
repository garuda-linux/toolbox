//! The Network page's root actions against the built helper: accepted in
//! a dry run, and refused whole when a module name could be an option.

mod common;
mod handmade;

use common::{TestResult, events, helper, outcomes, request};
use handmade::request_line;
use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{KernelModule, RootAction};

const BLACKLIST: &str = "/etc/modprobe.d/garuda-toolbox-blacklist.conf";

/// The dry run starts no program and writes no file, so this runs on any
/// machine and without root.
#[test]
fn module_actions_are_accepted_in_a_dry_run() -> TestResult {
    let module = KernelModule::new("toolbox_test_module")?;
    let before = std::fs::read_to_string(BLACKLIST).ok();
    let actions = vec![
        RootAction::LoadModule {
            module: module.clone(),
        },
        RootAction::UnloadModule {
            module: module.clone(),
        },
        RootAction::SetModuleBlacklisted {
            module,
            blacklisted: true,
        },
    ];

    let output = helper(&["apply"], &request(Mode::DryRun, actions)?)?;
    let skipped = outcomes(&output)?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(skipped, [Outcome::Skipped; 3]);
    assert_eq!(std::fs::read_to_string(BLACKLIST).ok(), before);
    Ok(())
}

#[test]
fn a_module_name_that_could_be_an_option_or_a_path_is_rejected() -> TestResult {
    for action in [
        r#"{"kind":"load-module","module":"--remove"}"#,
        r#"{"kind":"unload-module","module":"-a"}"#,
        r#"{"kind":"load-module","module":"../../tmp/evil"}"#,
        r#"{"kind":"set-module-blacklisted","module":"wl\ninstall wl /bin/sh","blacklisted":true}"#,
        r#"{"kind":"load-module","module":"wl nouveau"}"#,
    ] {
        let output = helper(&["apply"], &request_line("dry-run", action))?;
        let first = events(&output)?.into_iter().next();

        assert!(!output.status.success(), "{action}");
        assert!(
            matches!(&first, Some(Event::Rejected { reason }) if reason.contains("kernel module name")),
            "{action}: {}",
            output.stdout
        );
    }

    Ok(())
}
