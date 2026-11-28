use toolbox_cli::Exit;
use toolbox_cli::args::{Confirm, LocaleArgs, PackArgs};
use toolbox_core::{Snapshot, Toggle};

use super::{pack, switch};

fn locale(name: &str) -> LocaleArgs {
    LocaleArgs {
        locale: name.parse().unwrap(),
        confirm: Confirm::default(),
    }
}

#[test]
fn a_locale_this_system_does_not_offer_is_not_available() {
    let refused = switch(&locale("xx_XX.UTF-8"), Toggle::On, &Snapshot::default()).unwrap_err();
    assert_eq!(refused.exit(), Exit::Unavailable);
}

#[test]
fn a_package_that_is_no_pack_is_not_installed() {
    let args = PackArgs {
        package: "firefox".parse().unwrap(),
        confirm: Confirm::default(),
    };

    assert_eq!(
        pack(&args, Toggle::On, &Snapshot::default())
            .unwrap_err()
            .exit(),
        Exit::Unavailable
    );
}

/// The package, program and locale are plain names in JSON.
#[test]
fn a_pack_row_as_json_keeps_its_names_as_strings() {
    use toolbox_plan::{LocaleName, PackageName};

    use super::PackRow;

    let row = PackRow {
        package: PackageName::new("firefox-i18n-de").unwrap(),
        program: PackageName::new("firefox").unwrap(),
        locale: LocaleName::new("de_DE.UTF-8").unwrap(),
        installed: true,
    };

    assert_eq!(
        serde_json::to_string(&row).unwrap(),
        r#"{"id":"firefox-i18n-de","program":"firefox","locale":"de_DE.UTF-8","installed":true}"#
    );
}

#[test]
fn installed_packs_are_on() {
    use toolbox_cli::output::Mark;
    use toolbox_plan::{LocaleName, PackageName};

    use super::{PackRow, pack_listing};

    let row = |package: &str, installed| PackRow {
        package: PackageName::new(package).unwrap(),
        program: PackageName::new("firefox").unwrap(),
        locale: LocaleName::new("de_DE.UTF-8").unwrap(),
        installed,
    };

    let rows = [
        row("firefox-i18n-de", true),
        row("thunderbird-i18n-de", false),
    ];

    let listing = pack_listing(&rows);
    let marks: Vec<Mark> = listing.rows.iter().map(|shown| shown.mark).collect();

    assert_eq!(marks, [Mark::On, Mark::Off]);
    assert_eq!(listing.head, [""; 0]);
    assert_eq!(listing.summary.as_deref(), Some("2, 1 installed"));
    assert!(listing.hint.unwrap().contains("locales packs install"));
}

/// A locale change refused under `--json` is one JSON Lines error on
/// standard output, with the unavailable exit code.
#[test]
fn a_refused_locale_change_under_json_is_one_error_line() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let planned = switch(&locale("xx_XX.UTF-8"), Toggle::On, &Snapshot::default());
    let confirm = Confirm {
        yes: true,
        json: true,
        ..Confirm::default()
    };

    let mut out = Vec::new();
    let exit = crate::act::run(
        planned,
        confirm,
        None,
        &runtime,
        &mut crate::out::Out::new(&mut out, crate::out::Settings::default()),
    );

    let text = String::from_utf8(out).unwrap();
    assert_eq!(exit, Exit::Unavailable);
    assert!(
        text.starts_with(r#"{"schema":1,"event":"error","message":"#),
        "{text}"
    );
    assert_eq!(text.lines().count(), 1, "{text}");
}
