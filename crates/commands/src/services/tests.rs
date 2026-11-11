use toolbox_cli::output::{Paint, Style};

use super::text;
use crate::lists::ServiceRow;

fn bluetooth() -> ServiceRow {
    ServiceRow {
        name: "bluetooth.service".to_owned(),
        active: "active",
        state: "running".to_owned(),
        at_boot: true,
        description: "Bluetooth service".to_owned(),
    }
}

#[test]
fn a_service_shows_its_state_and_whether_it_starts_at_boot() {
    assert_eq!(
        text(Paint::new(Style::Plain), &bluetooth(), false),
        "bluetooth.service\n\n  \
         description  Bluetooth service\n  \
         state        active (running)\n  \
         at boot      yes\n  \
         service of   the system's\n"
    );
}

#[test]
fn a_user_service_says_it_is_yours() {
    let mut row = bluetooth();
    row.at_boot = false;
    let text = text(Paint::new(Style::Plain), &row, true);

    assert!(text.contains("at boot      no\n"), "{text}");
    assert!(text.contains("service of   yours\n"), "{text}");
}
