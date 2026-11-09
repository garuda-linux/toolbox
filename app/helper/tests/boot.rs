//! The Boot options page's root actions against the built helper:
//! accepted in a dry run without touching a file, and refused whole when a
//! value could leave its quotes, name another file or be an option.

mod common;
mod handmade;

use common::{TestResult, events, helper, outcomes, request};
use handmade::request_line;
use toolbox_plan::protocol::{Event, Mode, Outcome};
use toolbox_plan::{
    DefaultEntry, GrubSetting, GrubSettings, GrubTimeout, KernelParameters, PlymouthTheme,
    RootAction,
};

const GRUB_DEFAULTS: &str = "/etc/default/grub";

/// The dry run starts no program and writes no file, so this runs on any
/// machine and without root.
#[test]
fn boot_actions_are_accepted_in_a_dry_run() -> TestResult {
    let before = std::fs::read(GRUB_DEFAULTS).ok();
    let settings = GrubSettings::new(vec![
        GrubSetting::Default(DefaultEntry::Saved),
        GrubSetting::Timeout(GrubTimeout::new(3)?),
        GrubSetting::KernelParameters(
            KernelParameters::parse("quiet splash").ok_or("no parameters")?,
        ),
        GrubSetting::SaveDefault(true),
    ])?;

    let actions = vec![
        RootAction::SetGrubSettings { settings },
        RootAction::GenerateGrubConfig,
        RootAction::SetPlymouthTheme {
            theme: PlymouthTheme::new("bgrt")?,
        },
    ];

    let output = helper(&["apply"], &request(Mode::DryRun, actions)?)?;
    let skipped = outcomes(&output)?;

    assert!(output.status.success(), "{}", output.stderr);
    assert_eq!(skipped, vec![Outcome::Skipped; 3]);
    assert_eq!(std::fs::read(GRUB_DEFAULTS).ok(), before);
    Ok(())
}

#[test]
fn a_value_that_could_break_out_rejects_the_whole_request() -> TestResult {
    for action in [
        r#"{"kind":"set-grub-settings","settings":[{"key":"kernel-parameters","value":["quiet\"; touch /tmp/pwned; \""]}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"kernel-parameters","value":["x=$(id)"]}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"kernel-parameters","value":["a\nGRUB_DEFAULT=0"]}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"default","value":{"entry":"a`id`"}}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"background","value":"/etc/../root/a.png"}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"background","value":"/etc/shadow"}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"theme","value":"/a\"b/theme.txt"}]}"#,
        r#"{"kind":"set-grub-settings","settings":[{"key":"timeout","value":100000}]}"#,
        r#"{"kind":"set-grub-settings","settings":[]}"#,
        r#"{"kind":"set-plymouth-theme","theme":"--reset"}"#,
        r#"{"kind":"set-plymouth-theme","theme":"bgrt extra"}"#,
    ] {
        let output = helper(&["apply"], &request_line("dry-run", action))?;
        let first = events(&output)?.into_iter().next();

        assert!(!output.status.success(), "{action}");
        assert!(
            matches!(&first, Some(Event::Rejected { reason: _ })),
            "{action}: {}",
            output.stdout
        );
    }

    Ok(())
}
