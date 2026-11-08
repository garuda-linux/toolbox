use egui::accesskit::Role;
use egui::{Align2, Label, Rect, TextEdit, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::select::LIST_MAX_WIDTH;
use super::super::{
    ButtonKind, Select, Tone, banner, card, card_row, chips, highlighted, menu_action, menu_button,
    menu_item, number_field, pill_button, segmented, split_row, status_dot, switch, tag,
    text_field, title_line, trailing,
};
use crate::fonts::FontFile;
use crate::icons;
use crate::testing::{cap_off_centre, harness_with_font, painted_text, test_fonts, textured_rects};
use crate::theme::PILL_HEIGHT;

/// How far the middle of a label's capitals may sit from its control's
/// middle: half a pixel of rounding either way.
const CENTRED: f32 = 0.75;

const SHELL: &str = "/nix/store/rdxc7wg0r196bf6cicanfgcdkqhq9m9l-fish-4.9/bin/fish";

type Control = fn(&mut Ui);

/// `control` in `font`, in a card's row when `in_row`, else on the page.
fn drawn(font: Option<&FontFile>, in_row: bool, control: Control) -> Harness<'static> {
    let mut harness = harness_with_font(800.0, 400.0, font.cloned()).build_ui(move |ui| {
        if in_row {
            card(ui, |ui| card_row(ui, true, control));
        } else {
            control(ui);
        }
    });

    harness.run();
    harness
}

/// Runs `check` in every test font, in and outside a card's row.
fn everywhere(check: impl Fn(Option<&FontFile>, bool, &str)) {
    for font in test_fonts() {
        let name = font.as_ref().map_or("the bundled fonts", |font| &font.name);
        for in_row in [false, true] {
            check(font.as_ref(), in_row, &format!("{name}, in a row {in_row}"));
        }
    }
}

fn node_rect(harness: &Harness<'_>, role: Role, label: &str) -> Rect {
    harness.get_by_role_and_label(role, label).rect()
}

fn assert_centred(harness: &Harness<'_>, what: &str, text: &str, rect: Rect) {
    let off = cap_off_centre(harness, text, rect)
        .unwrap_or_else(|| panic!("{what}: {text} was not painted"));

    assert!(
        off.abs() < CENTRED,
        "{what}: the capitals of {text} sit {off} below the middle of {rect:?}"
    );
}

fn button(ui: &mut Ui) {
    pill_button(ui, "Remove", ButtonKind::Soft);
}

fn dropdown(ui: &mut Ui) {
    Select::new("Language", &["English", "Deutsch"])
        .selected(Some(0))
        .show(ui);
}

fn sort(ui: &mut Ui) {
    Select::new("Sort", &["Name", "Size"])
        .selected(Some(0))
        .caption("Sort by")
        .show(ui);
}

fn choice(ui: &mut Ui) {
    segmented(ui, "Theme", &[(0_u8, "Dark"), (1_u8, "Light")], 0_u8);
}

fn number(ui: &mut Ui) {
    let mut value = 5;
    number_field(ui, &mut value, 1..=10, 80.0, "Count");
}

fn field(ui: &mut Ui) {
    let mut text = "Hello".to_owned();
    let edit = TextEdit::singleline(&mut text)
        .desired_width(160.0)
        .align(Align2::LEFT_CENTER);

    let response = text_field(ui, edit);
    ui.ctx()
        .accesskit_node_builder(response.id, |node| node.set_label("Name"));
}

#[test]
fn the_text_of_every_control_is_centred_in_and_outside_a_row() {
    let controls: [(Control, &str, Role, &str); 6] = [
        (button, "Remove", Role::Button, "Remove"),
        (dropdown, "English", Role::ComboBox, "Language"),
        (sort, "Name", Role::ComboBox, "Sort"),
        (choice, "Dark", Role::RadioButton, "Dark"),
        (number, "5", Role::TextInput, "Count"),
        (field, "Hello", Role::TextInput, "Name"),
    ];

    everywhere(|font, in_row, what| {
        for (control, text, role, label) in controls {
            let harness = drawn(font, in_row, control);
            let rect = node_rect(&harness, role, label);
            assert_centred(&harness, &format!("{label}, {what}"), text, rect);
        }
    });
}

#[test]
fn a_dropdown_caption_and_chevron_sit_on_the_pill_middle() {
    everywhere(|font, in_row, what| {
        let harness = drawn(font, in_row, sort);
        let pill = node_rect(&harness, Role::ComboBox, "Sort");
        assert_centred(&harness, &format!("caption, {what}"), "Sort by", pill);

        let chevron = textured_rects(&harness)
            .into_iter()
            .find(|glyph| pill.contains_rect(*glyph))
            .expect("the chevron is painted on the pill");

        let off = chevron.center().y - pill.center().y;
        assert!(off.abs() < CENTRED, "{what}: the chevron sits {off} low");
    });
}

#[test]
fn a_chip_label_is_centred() {
    for font in test_fonts() {
        let harness = drawn(font.as_ref(), false, |ui| {
            chips(ui, &[(0_u8, "System"), (1_u8, "User")], 0_u8);
        });

        let chip = node_rect(&harness, Role::Button, "System");
        assert_centred(&harness, "chip", "System", chip);
    }
}

fn long_list(ui: &mut Ui) {
    Select::new("Login shell", &["Bash", SHELL])
        .placeholder("Pick a shell")
        .width(160.0)
        .show(ui);
}

/// `control` in `font` with its popup opened by a click on `role`
/// `label`.
fn opened(
    font: Option<&FontFile>,
    in_row: bool,
    control: Control,
    (role, label): (Role, &str),
) -> Harness<'static> {
    let mut harness = drawn(font, in_row, control);
    harness.get_by_role_and_label(role, label).click();
    harness.run();
    harness
}

#[test]
fn dropdown_entries_are_tall_enough_and_centred() {
    everywhere(|font, in_row, what| {
        let harness = opened(font, in_row, long_list, (Role::ComboBox, "Login shell"));
        let entry = node_rect(&harness, Role::ListBoxOption, "Bash");
        assert!(
            entry.height() >= PILL_HEIGHT - CENTRED,
            "{what}: a {} tall entry",
            entry.height()
        );
        assert_centred(&harness, &format!("entry, {what}"), "Bash", entry);
    });
}

#[test]
fn a_long_dropdown_entry_is_cut_inside_the_list() {
    let harness = opened(None, false, long_list, (Role::ComboBox, "Login shell"));
    let pill = node_rect(&harness, Role::ComboBox, "Login shell");
    let entry = node_rect(&harness, Role::ListBoxOption, SHELL);
    assert!(
        entry.width() > pill.width() && entry.width() <= LIST_MAX_WIDTH,
        "a {} wide entry under a {} wide button",
        entry.width(),
        pill.width()
    );

    let cut = painted_text(&harness, SHELL).expect("the long path is painted");
    assert!(cut.galley.elided, "the long path is not cut");
    let right = cut.pos.x + cut.galley.rect.right();
    assert!(
        right <= entry.right(),
        "the cut path ends at {right}, past the entry's {}",
        entry.right()
    );
}

fn menu(ui: &mut Ui) {
    menu_button(ui, "More", |ui| {
        menu_action(ui, Some(icons::EDIT_COPY), "Copy name");
        menu_item(ui, "Start");
    });
}

#[test]
fn menu_entries_and_their_icons_are_centred() {
    for font in test_fonts() {
        let harness = opened(font.as_ref(), false, menu, (Role::Button, "More"));
        for text in ["Copy name", "Start"] {
            let entry = node_rect(&harness, Role::MenuItem, text);
            assert_centred(&harness, "menu entry", text, entry);
        }

        let entry = node_rect(&harness, Role::MenuItem, "Copy name");
        let icon = textured_rects(&harness)
            .into_iter()
            .find(|glyph| entry.contains_rect(*glyph))
            .expect("the icon is painted in its entry");

        let off = icon.center().y - entry.center().y;
        assert!(off.abs() < CENTRED, "the icon sits {off} low");
    }
}

fn at_boot(ui: &mut Ui) {
    ui.add(Label::new("At boot").selectable(false));
    switch(ui, true, "Start at boot", true);
}

#[test]
fn a_caption_beside_a_switch_in_a_row_is_centred_on_it() {
    for font in test_fonts() {
        let harness = drawn(font.as_ref(), true, |ui| {
            split_row(
                ui,
                200.0,
                |ui| title_line(ui, false, |ui| ui.label("Bluetooth service")),
                at_boot,
            );
        });

        let switch = node_rect(&harness, Role::CheckBox, "Start at boot");
        assert_centred(&harness, "caption", "At boot", switch);
        assert_centred(&harness, "title", "Bluetooth service", switch);
    }
}

#[test]
fn a_caption_beside_a_switch_at_a_line_end_is_centred_on_it() {
    for font in test_fonts() {
        let harness = drawn(font.as_ref(), false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Console");
                trailing(ui, at_boot);
            });
        });

        let switch = node_rect(&harness, Role::CheckBox, "Start at boot");
        assert_centred(&harness, "caption", "At boot", switch);
    }
}

fn tagged(ui: &mut Ui) {
    // Wrapping, as the Components rows lay their title and tags out.
    title_line(ui, true, |ui| {
        ui.label("Power profiles service");
        status_dot(ui, "Active", Tone::Success);
        tag(ui, "Linked", Tone::Neutral);
    });
}

/// Where the ink of `text`, which has no descender, ends on its baseline.
fn baseline(harness: &Harness<'_>, text: &str) -> Option<f32> {
    painted_text(harness, text).map(|shape| shape.pos.y + shape.galley.mesh_bounds.bottom())
}

#[test]
fn a_tags_words_share_the_line_of_the_words_beside_it_and_sit_in_its_middle() {
    everywhere(|font, in_row, what| {
        let harness = drawn(font, in_row, tagged);

        let words = baseline(&harness, "Linked")
            .unwrap_or_else(|| panic!("{what}: the tag was not painted"));

        let beside = baseline(&harness, "Active")
            .unwrap_or_else(|| panic!("{what}: the status was not painted"));

        assert!(
            (words - beside).abs() < CENTRED,
            "{what}: the tag's words end at {words}, the status beside it at {beside}"
        );
        let tag = node_rect(&harness, Role::Label, "Linked");
        assert!(
            (tag.height() - 22.0).abs() < 0.5,
            "{what}: the tag is {} tall, not 22",
            tag.height()
        );
        assert_centred(&harness, &format!("tag, {what}"), "Linked", tag);
    });
}

#[test]
fn a_title_beside_a_button_shares_its_middle() {
    everywhere(|font, in_row, what| {
        let harness = drawn(font, in_row, |ui| {
            split_row(
                ui,
                120.0,
                |ui| title_line(ui, false, |ui| ui.label("Only when I ask")),
                button,
            );
        });

        let button = node_rect(&harness, Role::Button, "Remove");
        let title = format!("title, {what}");
        assert_centred(&harness, &title, "Only when I ask", button);
    });
}

#[test]
fn a_banner_line_sits_on_its_icon_middle() {
    for font in test_fonts() {
        let mut harness = harness_with_font(800.0, 200.0, font).build_ui(|ui| {
            banner(ui, Tone::Warning, "Could not read the mirrors", &[]);
        });

        harness.run();
        let icon = textured_rects(&harness)
            .first()
            .copied()
            .expect("the banner's icon is painted");

        assert_centred(&harness, "banner", "Could not read the mirrors", icon);
    }
}

fn highlighted_title(ui: &mut Ui) {
    split_row(
        ui,
        120.0,
        |ui| {
            // Wrapping, as the toggle and tweak rows lay their titles out.
            title_line(ui, true, |ui| {
                let font = crate::fonts::font(ui.style(), crate::fonts::Weight::SemiBold, 1.0);
                let job = highlighted(ui, "Ad blocking", "block", font, egui::Color32::WHITE);
                ui.label(job);
            });
        },
        |ui| {
            switch(ui, false, "Ad blocking", true);
        },
    );
}

#[test]
fn a_searched_title_is_centred_on_its_switch() {
    everywhere(|font, in_row, what| {
        let harness = drawn(font, in_row, highlighted_title);
        let switch = node_rect(&harness, Role::CheckBox, "Ad blocking");
        assert_centred(&harness, &format!("title, {what}"), "Ad blocking", switch);
    });
}
