use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;

use super::{Refusals, page, tab};

#[test]
fn a_host_that_cannot_be_used_is_told_as_a_problem() {
    let mut harness = page(Refusals::default());
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
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Tools");

    let widths: Vec<f32> = [
        "Number of pings",
        "Timeout in seconds",
        "Most hops (for traceroute)",
    ]
    .map(|name| {
        harness
            .get_by_role_and_label(Role::TextInput, name)
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
    let mut harness = page(Refusals::default());
    tab(&mut harness, "Tools");

    let field = harness
        .get_by_role_and_label(Role::TextInput, "Number of pings")
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
    let mut harness = page(Refusals::default());
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
