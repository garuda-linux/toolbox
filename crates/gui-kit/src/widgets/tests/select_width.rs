use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::Select;
use crate::testing::{harness, painted_text};

const LONG: &str = "Deutsch (Deutschland), de_DE.UTF-8";
const LOCALES: [&str; 40] = [
    "af_ZA", "ar_EG", "bg_BG", "ca_ES", "cs_CZ", "da_DK", "de_AT", "de_CH", "de_DE", "el_GR",
    "en_AU", "en_CA", "en_GB", "en_IE", "en_US", "es_ES", "es_MX", "et_EE", "fi_FI", "fr_CA",
    "fr_FR", "he_IL", "hr_HR", "hu_HU", "id_ID", "it_IT", "ja_JP", "ko_KR", "lt_LT", "lv_LV",
    "nb_NO", "nl_NL", "pl_PL", "pt_BR", "pt_PT", "ro_RO", "ru_RU", "sk_SK", "sv_SE", "tr_TR",
];

fn shown(options: &'static [&'static str], chosen: usize) -> Harness<'static, f32> {
    let mut harness = harness(900.0, 120.0).build_ui_state(
        move |ui, width: &mut f32| {
            *width = Select::fitting_width(ui, options.iter().copied());
            _ = Select::new("Language", options)
                .selected(Some(chosen))
                .width(*width)
                .show(ui);
        },
        0.0,
    );

    harness.run();
    harness
}

#[test]
fn a_fitting_width_shows_the_longest_option_whole() {
    let harness = shown(&["English", LONG], 1);
    let text = painted_text(&harness, LONG).expect("the option is painted");

    assert!(!text.galley.elided, "cut at {}", harness.state());
}

#[test]
fn a_fitting_width_stops_at_the_widest_open_list() {
    let narrow = shown(&["English", "x"], 0);
    let short = *narrow.state();

    let harness = shown(
        &[
            LONG,
            "a name far too long for any dropdown to hold, a name far too long for any dropdown to hold",
        ],
        0,
    );

    assert!(short < 100.0, "{short}");
    assert!((harness.state() - super::super::select::LIST_MAX_WIDTH).abs() < 0.5);
}

/// A dropdown of more options than the window holds, `chosen` far down.
fn long_list(chosen: usize) -> Harness<'static> {
    let mut harness = harness(400.0, 300.0).build_ui(move |ui| {
        _ = Select::new("Locale", &LOCALES)
            .selected(Some(chosen))
            .show(ui);
    });

    harness.run();
    harness
}

fn window() -> egui::Rect {
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400.0, 300.0))
}

#[test]
fn a_long_list_opens_on_its_chosen_option() {
    let mut harness = long_list(35);

    harness
        .get_by_role_and_label(Role::ComboBox, "Locale")
        .click();
    harness.run();

    let chosen = harness
        .get_by_role_and_label(Role::ListBoxOption, LOCALES[35])
        .rect();

    assert!(window().contains_rect(chosen), "{chosen:?}");
}

#[test]
fn a_long_list_stays_inside_the_window() {
    let mut harness = long_list(0);

    harness
        .get_by_role_and_label(Role::ComboBox, "Locale")
        .click();
    harness.run();

    let first = harness
        .get_by_role_and_label(Role::ListBoxOption, LOCALES[0])
        .rect();

    assert!(window().contains_rect(first), "{first:?}");
    assert!(
        painted_text(&harness, LOCALES[39]).is_none_or(|text| !window().contains(text.pos)),
        "the list shows all forty options at once"
    );
}

#[test]
fn the_pill_tells_screen_readers_whether_its_list_is_open() {
    let mut harness = long_list(0);
    let expanded = |shown: &Harness<'_>| {
        shown
            .get_by_role_and_label(Role::ComboBox, "Locale")
            .accesskit_node()
            .data()
            .is_expanded()
    };

    assert_eq!(expanded(&harness), Some(false));

    harness
        .get_by_role_and_label(Role::ComboBox, "Locale")
        .click();
    harness.run();

    assert_eq!(expanded(&harness), Some(true));
}

#[test]
fn the_down_arrow_walks_into_the_open_list() {
    let mut harness = long_list(0);

    harness.key_press(egui::Key::Tab);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();

    assert!(
        harness
            .get_by_role_and_label(Role::ListBoxOption, LOCALES[0])
            .is_focused()
    );
}
