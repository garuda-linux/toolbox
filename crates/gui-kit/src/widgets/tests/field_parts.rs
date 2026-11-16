use egui::accesskit::Role;
use egui::{Align2, TextEdit, Theme};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::{SecretText, SecretTexts, forget_text, secret_field, text_field};
use crate::testing::{click_at, filled_rect, harness, painted_text, rim_colours_around};
use crate::tokens;

const SECRET: SecretTexts<'static> = SecretTexts {
    name: "Password",
    show: "Show password",
    hide: "Hide password",
};

fn plain_field() -> Harness<'static, String> {
    let mut harness = harness(400.0, 100.0)
        .with_theme(Theme::Dark)
        .build_ui_state(
            |ui, text: &mut String| {
                let edit = TextEdit::singleline(text)
                    .desired_width(200.0)
                    .align(Align2::LEFT_CENTER);

                let response = text_field(ui, edit);
                ui.ctx()
                    .accesskit_node_builder(response.id, |node| node.set_label("Name"));
            },
            String::new(),
        );

    harness.run();
    harness
}

/// A click focuses a field as the keyboard does, and the ring shows
/// where the typing goes either way.
#[test]
fn a_field_focused_by_a_click_shows_its_ring() {
    let mut harness = plain_field();
    let field = harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .rect();

    click_at(&mut harness, field.center());

    let ring = tokens::current(&harness.ctx).accent_ring;
    assert!(rim_colours_around(&harness, field.center()).contains(&ring));
}

#[test]
fn a_field_under_the_pointer_steps_its_fill_towards_the_focused_one() {
    let mut harness = plain_field();
    let field = harness
        .get_by_role_and_label(Role::TextInput, "Name")
        .rect();

    harness.hover_at(field.center());
    harness.run();

    let tokens = tokens::current(&harness.ctx);
    let step = tokens.field.lerp_to_gamma(tokens.hover, 0.5);
    assert!(filled_rect(&harness, step).is_some(), "no hover fill");
    assert!(filled_rect(&harness, tokens.field).is_none());
}

fn secret() -> Harness<'static, (SecretText, Option<egui::Id>)> {
    let mut harness = harness(400.0, 100.0).build_ui_state(
        |ui, (text, id): &mut (SecretText, Option<egui::Id>)| {
            *id = Some(secret_field(ui, text, SECRET).id);
        },
        (SecretText::with_room(64), None),
    );

    harness.run();
    harness
        .get_by_role_and_label(Role::PasswordInput, "Password")
        .focus();
    harness.run();
    harness
        .get_by_role_and_label(Role::PasswordInput, "Password")
        .type_text("hunter2");
    harness.run();
    harness
}

#[test]
fn the_eye_shows_the_password_and_hides_it_again() {
    let mut harness = secret();
    assert!(
        painted_text(&harness, "hunter2").is_none(),
        "shown at first"
    );

    harness
        .get_by_role_and_label(Role::Button, "Show password")
        .click();
    harness.run();
    assert!(painted_text(&harness, "hunter2").is_some(), "still hidden");

    harness
        .get_by_role_and_label(Role::Button, "Hide password")
        .click();
    harness.run();
    assert!(painted_text(&harness, "hunter2").is_none(), "still shown");
}

#[test]
fn forgetting_the_field_hides_a_shown_password() {
    let mut harness = secret();
    harness
        .get_by_role_and_label(Role::Button, "Show password")
        .click();
    harness.run();

    let id = harness.state().1.expect("the field showed");
    forget_text(&harness.ctx, id);
    harness.run();

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Show password")
            .is_some()
    );
}
