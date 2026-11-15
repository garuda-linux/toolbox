mod schemes;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_config::{LogLevel, RefreshInterval, Settings};

use super::SettingsPage;
use crate::command::Command;
use crate::settings::{SettingsCommand, ThemeChoice};
use crate::testing::{PageFixture, PageState};

fn page(settings: Settings) -> Harness<'static, PageState<SettingsPage>> {
    let mut harness = PageFixture {
        settings,
        ..PageFixture::default()
    }
    .harness(SettingsPage::default());

    harness.run();
    harness
}

fn sent(harness: &Harness<'static, PageState<SettingsPage>>) -> Vec<Command> {
    harness.state().fixture.sent()
}

#[test]
fn the_three_sections_show_their_rows() {
    let harness = page(Settings::default());

    for label in [
        "Appearance",
        "Behaviour",
        "Diagnostics",
        "Language",
        "Theme",
        "Reduced motion",
        "Start at login",
        "Check for updates automatically",
        "Time between checks",
        "Services page opens on",
        "Show AUR packages",
        "Copy reports to the clipboard",
        "Log file detail",
    ] {
        assert!(
            harness.query_all_by_label(label).next().is_some(),
            "{label}"
        );
    }
}

#[test]
fn a_theme_segment_picks_the_theme() {
    let mut harness = page(Settings::default());

    harness
        .get_by_role_and_label(Role::RadioButton, "Dark")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(SettingsCommand::Theme(ThemeChoice::Dark))]
    );
}

#[test]
fn minutes_outside_the_limits_are_held_to_them_with_a_note() {
    let mut harness = page(Settings::default());
    let field = || "Minutes between checks";

    harness
        .get_by_role_and_label(Role::TextInput, field())
        .focus();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::COMMAND, Key::A);
    harness
        .get_by_role_and_label(Role::TextInput, field())
        .type_text("2");
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(SettingsCommand::Refresh(
            RefreshInterval::Minutes(5)
        ))]
    );
    assert!(
        harness
            .query_by_label_contains("so this is 5 minutes")
            .is_some()
    );
}

#[test]
fn turning_the_switch_off_stops_the_checks() {
    let mut harness = page(Settings::default());

    harness
        .get_by_role_and_label(Role::CheckBox, "Check for updates automatically")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(SettingsCommand::Refresh(
            RefreshInterval::Off
        ))]
    );
}

#[test]
fn start_at_login_switches_the_autostart() {
    let mut harness = page(Settings::default());

    harness
        .get_by_role_and_label(Role::CheckBox, "Start at login")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(SettingsCommand::StartAtLogin(true))]
    );
}

#[test]
fn the_log_detail_is_picked_from_a_list() {
    let mut harness = page(Settings::default());

    // The last row lies below the window at first.
    harness
        .get_by_role_and_label(Role::CheckBox, "Copy reports to the clipboard")
        .scroll_to_me();
    harness.run();
    harness
        .get_by_role_and_label(Role::ComboBox, "Log file detail")
        .click();
    harness.run();
    harness.get_by_label("Detailed").click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(SettingsCommand::LogLevel(LogLevel::Debug))]
    );
}

#[test]
fn each_section_says_what_it_holds_under_its_heading_without_a_help_button() {
    let harness = page(Settings::default());

    let heading = harness.get_by_label("Appearance").rect();
    let help = harness
        .get_by_label("The language, colours and motion of this window.")
        .rect();

    assert!(
        help.top() >= heading.bottom(),
        "the help sits under its heading"
    );
    assert!(harness.query_by_label("Help for Appearance").is_none());
}

#[test]
fn the_minutes_show_under_the_switch_only_while_the_checks_are_on() {
    let on = page(Settings::default());
    let switch = on
        .get_by_role_and_label(Role::CheckBox, "Check for updates automatically")
        .rect();

    let minutes = on
        .get_by_role_and_label(Role::TextInput, "Minutes between checks")
        .rect();

    assert!(switch.bottom() <= minutes.top());

    let mut settings = Settings::default();
    settings.status.set(RefreshInterval::Off);
    let off = page(settings);

    assert!(
        off.query_by_role_and_label(Role::TextInput, "Minutes between checks")
            .is_none()
    );
    assert!(off.query_by_label("Time between checks").is_none());
}

#[test]
fn the_minutes_and_their_unit_sit_centred_in_the_field_as_one_group() {
    let harness = page(Settings::default());
    let field = harness
        .get_by_role_and_label(Role::TextInput, "Minutes between checks")
        .rect();

    let painted = |text| {
        let shape = toolbox_gui_kit::testing::painted_text_within(&harness, text, field)
            .unwrap_or_else(|| panic!("{text} is not inside {field:?}"));

        shape.galley.rect.translate(shape.pos.to_vec2())
    };

    let digits = painted("60");
    let unit = painted("min");
    assert!(digits.right() < unit.left(), "{digits:?} then {unit:?}");
    let group = f32::midpoint(digits.left(), unit.right());
    assert!(
        (group - field.center().x).abs() < 1.0,
        "the group's middle {group}, the field's {}",
        field.center().x
    );
    assert!(
        (field.center().y - unit.center().y).abs() < 1.0,
        "field centre {} against unit centre {}",
        field.center().y,
        unit.center().y,
    );
}

/// A pointer click at `pos`, where kittest's own click would need a
/// widget under it.
fn click_at(harness: &mut Harness<'static, PageState<SettingsPage>>, pos: egui::Pos2) {
    let button = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };

    harness.event(egui::Event::PointerMoved(pos));
    harness.step();
    harness.event(button(true));
    harness.step();
    harness.event(button(false));
    harness.run();
}

#[test]
fn a_click_on_a_switch_rows_words_flips_it() {
    let mut harness = page(Settings::default());
    let words = harness
        .get_by_role_and_label(Role::Label, "Start at login")
        .rect()
        .center();

    click_at(&mut harness, words);

    assert_eq!(
        sent(&harness),
        [Command::from(SettingsCommand::StartAtLogin(true))]
    );
}

#[test]
fn turned_on_again_the_checks_keep_their_last_minutes() {
    let mut settings = Settings::default();
    settings.status.set(RefreshInterval::Minutes(30));
    settings.status.set(RefreshInterval::Off);
    let mut harness = page(settings);

    assert!(
        harness
            .query_all_by_label_contains("only when you click Read again")
            .next()
            .is_some()
    );

    harness
        .get_by_role_and_label(Role::CheckBox, "Check for updates automatically")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::from(SettingsCommand::Refresh(
            RefreshInterval::Minutes(30)
        ))]
    );
}

#[test]
fn the_aur_switch_turns_the_aur_list_on() {
    let mut harness = page(Settings::default());

    harness
        .get_by_role_and_label(Role::CheckBox, "Show AUR packages")
        .scroll_to_me();
    harness.run();
    harness
        .get_by_role_and_label(Role::CheckBox, "Show AUR packages")
        .click();
    harness.run();

    assert_eq!(sent(&harness), [Command::from(SettingsCommand::Aur(true))]);
}

#[test]
fn the_longest_minutes_fit_their_field_in_english_and_german() {
    for (language, name, unit) in [
        ("en-US", "Minutes between checks", "min"),
        ("de-DE", "Prüfintervall in Minuten", "Min."),
    ] {
        let mut settings = Settings::default();
        settings.status.refresh = RefreshInterval::Minutes(toolbox_config::MAX_MINUTES);

        let mut harness = page(settings);
        let language = language.parse().expect("a language tag");
        crate::i18n::set_language(&harness.ctx, &language);
        harness.run();

        let field = harness.get_by_role_and_label(Role::TextInput, name).rect();
        let padding = harness.ctx.global_style().spacing.button_padding.x;
        let painted = |text| {
            let shape = toolbox_gui_kit::testing::painted_text_within(&harness, text, field)
                .unwrap_or_else(|| panic!("{language}: {text} is not inside {field:?}"));

            shape.galley.rect.translate(shape.pos.to_vec2())
        };

        let digits = painted("1440");
        let unit = painted(unit);

        assert!(
            digits.left() >= field.left() + padding - 0.5,
            "{language}: {digits:?} in {field:?}"
        );
        assert!(
            unit.left() - digits.right() >= 5.5,
            "{language}: {digits:?} then {unit:?}"
        );
    }
}
