use toolbox_cli::output::{Mark, Row};

use super::super::{KernelRow, LocaleRow, ServiceRow, ToggleRow, UserRow};
use super::{kernels, locales, services, toggles, users};

fn service(name: &str, active: &'static str, at_boot: bool) -> ServiceRow {
    ServiceRow {
        name: name.to_owned(),
        active,
        state: "running".to_owned(),
        at_boot,
        description: String::new(),
    }
}

#[test]
fn services_are_marked_by_their_state_and_counted() {
    let rows = [
        service("a.service", "active", true),
        service("b.service", "inactive", true),
        service("c.service", "failed", false),
        service("d.service", "activating", false),
    ];

    let listing = services(&rows, false);

    assert_eq!(listing.title, "Services");
    assert_eq!(listing.head, [""; 0]);
    assert_eq!(listing.summary.as_deref(), Some("4, 1 running, 1 failed"));
    assert_eq!(
        listing.rows,
        [
            Row::cells(Mark::On, &["a.service", "running", "at boot"]),
            Row::cells(Mark::Off, &["b.service", "stopped", "at boot"]),
            Row::cells(Mark::Problem, &["c.service", "failed", ""]),
            Row::cells(Mark::Blank, &["d.service", "activating", ""]),
        ]
    );
    assert_eq!(
        listing.hint.as_deref(),
        Some("garuda-toolbox services start <name>")
    );
}

#[test]
fn a_user_service_hint_keeps_user() {
    let rows = [service("a.service", "inactive", false)];
    let listing = services(&rows, true);

    assert_eq!(listing.summary.as_deref(), Some("1, 0 running"));
    assert_eq!(
        listing.hint.as_deref(),
        Some("garuda-toolbox services start --user <name>")
    );
}

fn toggle(id: &str, state: &'static str) -> ToggleRow {
    ToggleRow {
        page: "components",
        section: "containers".to_owned(),
        id: id.to_owned(),
        name: id.to_uppercase(),
        state,
        available: true,
    }
}

#[test]
fn toggles_are_marked_on_off_and_unknown() {
    let rows = [
        toggle("a", "on"),
        toggle("b", "off"),
        toggle("c", "unknown"),
    ];

    let listing = toggles(&rows, "Switches", "garuda-toolbox toggle on <id>");

    assert_eq!(listing.summary.as_deref(), Some("3, 1 on, 1 unknown"));
    assert_eq!(
        listing.rows,
        [
            Row::cells(Mark::On, &["a", "A"]),
            Row::cells(Mark::Off, &["b", "B"]),
            Row::cells(Mark::Unknown, &["c", "C"]),
        ]
    );
    assert_eq!(
        listing.hint.as_deref(),
        Some("garuda-toolbox toggle on <id>")
    );
}

fn kernel(name: &str, installed: bool, running: bool) -> KernelRow {
    KernelRow {
        name: name.to_owned(),
        version: "6.1".to_owned(),
        installed,
        running,
        headers: false,
    }
}

#[test]
fn kernels_say_which_runs_and_which_are_installed() {
    let rows = [
        kernel("linux", true, true),
        kernel("linux-lts", true, false),
        kernel("linux-zen", false, false),
    ];

    let listing = kernels(&rows);

    assert_eq!(
        listing.summary.as_deref(),
        Some("3, 2 installed, 1 running")
    );
    assert_eq!(
        listing.rows,
        [
            Row::cells(Mark::On, &["linux", "6.1", "running"]),
            Row::cells(Mark::On, &["linux-lts", "6.1", "installed"]),
            Row::cells(Mark::Off, &["linux-zen", "6.1", ""]),
        ]
    );
}

#[test]
fn locales_mark_the_enabled_ones_and_name_the_system_one() {
    let row = |locale: &str, enabled, system| LocaleRow {
        locale: locale.to_owned(),
        charset: "UTF-8".to_owned(),
        name: None,
        enabled,
        system,
    };

    let rows = [
        row("en_US.UTF-8", true, true),
        row("de_DE.UTF-8", true, false),
        row("fr_FR.UTF-8", false, false),
    ];

    let listing = locales(&rows);

    assert_eq!(listing.summary.as_deref(), Some("3, 2 enabled"));
    assert_eq!(
        listing.rows,
        [
            Row::cells(Mark::On, &["en_US.UTF-8", "UTF-8", "system"]),
            Row::cells(Mark::On, &["de_DE.UTF-8", "UTF-8", "enabled"]),
            Row::cells(Mark::Off, &["fr_FR.UTF-8", "UTF-8", ""]),
        ]
    );
}

#[test]
fn hidden_accounts_are_counted_and_all_is_hinted() {
    let ada = UserRow {
        name: "ada".to_owned(),
        uid: 1000,
        full_name: "Ada".to_owned(),
        home: "/home/ada".to_owned(),
        shell: "/bin/fish".to_owned(),
    };

    let listing = users(std::slice::from_ref(&ada), 31);

    assert_eq!(listing.head, ["USER", "NAME", "SHELL"]);
    assert_eq!(
        listing.summary.as_deref(),
        Some("1 shown, 31 system accounts hidden")
    );
    assert_eq!(
        listing.rows,
        [Row::cells(Mark::Blank, &["ada", "Ada", "/bin/fish"])]
    );
    assert!(listing.hint.unwrap().contains("--all"));

    let every = users(std::slice::from_ref(&ada), 0);
    assert_eq!(every.summary.as_deref(), Some("1"));
    assert!(every.hint.is_none());
}
