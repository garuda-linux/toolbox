use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_plan::{HostName, PingCount, PingTimeout};

use super::{Shown, page, sent, tab};
use crate::command::{Command, NetworkCommand, UiCommand};

const FINISHED: &str = "Ping finished. Its output is in the Console.";

/// Clicks Run ping, then ends the next plan as `succeeded` says.
fn pinged(succeeded: bool, console_open: bool) -> Shown {
    let mut harness = page();
    tab(&mut harness, "Tools");
    harness.state_mut().fixture.console_open = console_open;
    harness
        .get_by_role_and_label(Role::Button, "Run ping")
        .click();
    harness.run();

    assert!(
        harness.query_by_label(FINISHED).is_none(),
        "nothing ended yet"
    );

    let plans = &mut harness.state_mut().fixture.plans;
    plans.finished += 1;
    plans.last_succeeded = succeeded;
    harness.run();
    harness
}

#[test]
fn an_ended_ping_says_so_and_offers_the_console() {
    let mut harness = pinged(true, false);

    assert!(harness.query_by_label_contains(FINISHED).is_some());
    harness
        .get_by_role_and_label(Role::Button, "Show Console")
        .click();
    harness.run();

    let ping = NetworkCommand::Ping {
        host: HostName::new("garudalinux.org").unwrap(),
        count: PingCount::new(4).unwrap(),
        timeout: PingTimeout::new(5).unwrap(),
    };

    assert_eq!(
        sent(&harness),
        [Command::from(ping), Command::from(UiCommand::ToggleConsole)]
    );
}

#[test]
fn a_failed_ping_says_so_and_an_open_console_is_not_offered() {
    let harness = pinged(false, true);

    assert!(
        harness
            .query_by_label_contains("Ping ended with an error. The Console shows why.")
            .is_some()
    );
    assert!(harness.query_by_label("Show Console").is_none());
}

#[test]
fn a_host_that_cannot_be_used_is_told_as_a_problem() {
    let mut harness = page();
    tab(&mut harness, "Tools");

    let field = harness.get_by_role_and_label(Role::TextInput, "Host");
    field.click();
    harness.run();
    for _ in 0..20_u8 {
        harness.key_press(Key::Backspace);
    }

    harness
        .get_by_role_and_label(Role::TextInput, "Host")
        .type_text("not a host");
    harness.run();

    let problem = "Enter a host name or an IP address, for example garudalinux.org.";
    let warning = toolbox_gui_kit::tokens::current(&harness.ctx).warning;
    assert_eq!(
        toolbox_gui_kit::testing::text_colour(&harness, problem),
        Some(warning)
    );
}

#[test]
fn the_number_fields_are_equally_wide() {
    let mut harness = page();
    tab(&mut harness, "Tools");

    let widths: Vec<f32> = [
        "Number of pings",
        "Timeout in seconds",
        "Most hops (for traceroute)",
    ]
    .map(|name| {
        harness
            .get_by_role_and_label(Role::SpinButton, name)
            .rect()
            .width()
    })
    .to_vec();

    assert!(
        widths
            .windows(2)
            .all(|pair| matches!(pair, [first, second] if (first - second).abs() < 0.5)),
        "{widths:?}"
    );
}

#[test]
fn a_number_ends_at_the_right_of_its_field() {
    let mut harness = page();
    tab(&mut harness, "Tools");

    let field = harness
        .get_by_role_and_label(Role::SpinButton, "Number of pings")
        .rect();

    let digits = toolbox_gui_kit::testing::painted_text(&harness, "4").expect("painted");
    let end = digits.pos.x + digits.galley.rect.right();
    let padding = harness.ctx.global_style().spacing.button_padding.x;

    assert!(
        (field.right() - padding - end).abs() < 1.5,
        "the number ends at {end}, its field at {}",
        field.right()
    );
}

#[test]
fn the_host_sits_in_the_middle_of_its_field() {
    let mut harness = page();
    tab(&mut harness, "Tools");

    let field = harness
        .get_by_role_and_label(Role::TextInput, "Host")
        .rect();

    let host = toolbox_gui_kit::testing::painted_text(&harness, "garudalinux.org")
        .expect("the host is painted");

    let middle = host.pos.y + host.galley.rect.center().y;

    assert!(
        (field.center().y - middle).abs() < 1.5,
        "the host's middle is at {middle}, its field's at {}",
        field.center().y
    );
}
