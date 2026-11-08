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

#[test]
fn installed_packs_are_on() {
    use toolbox_cli::output::Mark;

    use super::{PackRow, pack_listing};

    let row = |package: &str, installed| PackRow {
        package: package.to_owned(),
        program: "firefox".to_owned(),
        locale: "de_DE.UTF-8".to_owned(),
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
