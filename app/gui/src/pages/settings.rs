mod minutes;
mod schemes;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use egui::{Context, Ui};
use toolbox_catalog::SchemeMode;
use toolbox_config::{LogLevel, RefreshInterval};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{Select, card, section_intro, segmented, segmented_width};

use self::minutes::MinutesField;
use super::header::title_header;
use super::setting_row::{Words, row, switch_row};
use super::{PageContext, PageView};
use crate::i18n;
use crate::i18n::Arg;
use crate::settings::{MotionChoice, SettingsCommand, SettingsView, ThemeChoice};

const NARROW_CONTROL: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_narrow;
/// The minutes field with its unit inside: "1440 Min." needs about 92.
const MINUTES_WIDTH: f32 = 96.0;

#[derive(Debug, Default)]
pub struct SettingsPage {
    minutes: MinutesField,
}

struct Texts {
    title: Cow<'static, str>,
    description: Cow<'static, str>,
}

type Send<'a> = &'a dyn Fn(SettingsCommand);

impl PageView for SettingsPage {
    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        title_header(ui, "page-settings", "page-settings-subtitle");
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("settings ui").entered();
        let view = context.preferences;
        let send = |command: SettingsCommand| context.commands.send(command);

        section(
            ui,
            "settings-appearance",
            icons::PREFERENCES_DESKTOP_THEME_GLOBAL,
        );
        card(ui, |ui| {
            language(ui, &view, &send);
            theme(ui, &view, &send);
            let schemes = (
                &context.catalogs.colour_schemes,
                &view.settings.colours,
                view.desktop,
            );

            schemes::scheme(ui, SchemeMode::Dark, schemes, &send);
            schemes::scheme(ui, SchemeMode::Light, schemes, &send);
            motion(ui, &view, &send);
        });
        ui.add_space(ui.tokens().sizes.section_gap);

        section(ui, "settings-behaviour", icons::CHRONOMETER);
        card(ui, |ui| {
            autostart(ui, &view, &send);
            if automatic(ui, &view, &send) {
                self.interval(ui, &view, &send);
            }

            user_units(ui, &view, &send);

            aur(ui, &view, &send);
        });
        ui.add_space(ui.tokens().sizes.section_gap);

        section(ui, "settings-diagnostics", icons::OFFICE_REPORT);
        card(ui, |ui| {
            copy_reports(ui, &view, &send);
            log_level(ui, &view, &send);
        });
    }
}

impl SettingsPage {
    /// Drawn only while the checks are on, so no row waits greyed out.
    fn interval(&mut self, ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
        let ctx = ui.ctx().clone();
        let held = self.minutes.held().map(|kept| {
            i18n::text_with(
                &ctx,
                "settings-refresh-clamped",
                &[("minutes", Arg::from(kept))],
            )
        });

        let words = Words {
            current: None,
            title: &i18n::text(&ctx, "settings-refresh-interval"),
            description: &i18n::text(&ctx, "settings-refresh-interval-description"),
            note: held.as_deref(),
            queued: None,
        };

        let name = i18n::text(&ctx, "settings-refresh-minutes");
        let unit = i18n::text(&ctx, "settings-refresh-unit");
        let minutes = view.settings.status.minutes();
        let field = &mut self.minutes;
        row(ui, false, &words, MINUTES_WIDTH, |ui| {
            let texts = (name.as_ref(), unit.as_ref());
            if let Some(interval) = field.show(ui, minutes, texts, MINUTES_WIDTH) {
                send(SettingsCommand::Refresh(interval));
            }
        });
    }
}

impl Texts {
    fn borrowed(&self) -> Words<'_> {
        Words {
            current: None,
            title: &self.title,
            description: &self.description,
            note: None,
            queued: None,
        }
    }
}

/// A section's heading with what the section holds under it, in sight
/// rather than behind a button.
fn section(ui: &mut Ui, id: &str, icon: Icon) {
    let ctx = ui.ctx().clone();
    let help = i18n::text(&ctx, &format!("{id}-help"));
    section_intro(ui, icon, &i18n::text(&ctx, id), &help);
}

fn language(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let offered = i18n::available();
    let codes: Vec<Option<String>> = std::iter::once(None)
        .chain(offered.iter().map(|language| Some(language.to_string())))
        .collect();

    let system = i18n::text(&ctx, "settings-language-system").into_owned();
    let names: Vec<String> = std::iter::once(system)
        .chain(offered.iter().map(i18n::name_of))
        .collect();

    let shown: Vec<&str> = names.iter().map(String::as_str).collect();
    let current = codes
        .iter()
        .position(|code| code.as_deref() == view.language);

    let words = texts(&ctx, "settings-language");
    row(ui, true, &words.borrowed(), NARROW_CONTROL, |ui| {
        let picked = Select::new(&words.title, &shown)
            .selected(current)
            .width(NARROW_CONTROL)
            .show(ui);

        if let Some(code) = picked.value.and_then(|index| codes.get(index)) {
            send(SettingsCommand::Language(code.clone()));
        }
    });
}

fn theme(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let options = [
        (
            ThemeChoice::FollowDesktop,
            i18n::text(&ctx, "settings-theme-desktop"),
        ),
        (ThemeChoice::Dark, i18n::text(&ctx, "settings-theme-dark")),
        (ThemeChoice::Light, i18n::text(&ctx, "settings-theme-light")),
    ];

    let words = texts(&ctx, "settings-theme");
    choice(ui, &words, &options, view.theme, |picked| {
        send(SettingsCommand::Theme(picked));
    });
}

fn motion(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let options = [
        (
            MotionChoice::FollowSystem,
            i18n::text(&ctx, "settings-motion-system"),
        ),
        (
            MotionChoice::Reduced,
            i18n::text(&ctx, "settings-motion-reduced"),
        ),
        (MotionChoice::Full, i18n::text(&ctx, "settings-motion-full")),
    ];

    let words = texts(&ctx, "settings-motion");
    choice(ui, &words, &options, view.motion, |picked| {
        send(SettingsCommand::Motion(picked));
    });
}

fn autostart(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let words = texts(&ctx, "settings-autostart");
    if switch_row(ui, true, &words.borrowed(), view.autostart, true) {
        send(SettingsCommand::StartAtLogin(!view.autostart));
    }
}

/// Whether the checks are on after this frame's click, so the interval row
/// follows the switch at once. Turned back on, the kept minutes return.
fn automatic(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) -> bool {
    let ctx = ui.ctx().clone();
    let status = &view.settings.status;
    let words = texts(&ctx, "settings-refresh-auto");

    let on = status.refresh != RefreshInterval::Off;
    if switch_row(ui, false, &words.borrowed(), on, true) {
        send(SettingsCommand::Refresh(status.turned(!on)));
        return !on;
    }

    on
}

fn user_units(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let options = [
        (false, i18n::text(&ctx, "settings-user-units-system")),
        (true, i18n::text(&ctx, "settings-user-units-user")),
    ];

    let words = texts(&ctx, "settings-user-units");
    let current = view.settings.services.show_user_units;
    choice(ui, &words, &options, current, |user| {
        send(SettingsCommand::UserUnits(user));
    });
}

/// Off by default: the list is a download from the AUR, and its
/// packages are community scripts.
fn aur(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let words = texts(&ctx, "settings-aur");
    let shown = view.settings.packages.show_aur;
    if switch_row(ui, false, &words.borrowed(), shown, true) {
        send(SettingsCommand::Aur(!shown));
    }
}

fn copy_reports(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let words = texts(&ctx, "settings-copy-reports");
    let copy = view.settings.diagnostics.copy_to_clipboard;
    if switch_row(ui, true, &words.borrowed(), copy, true) {
        send(SettingsCommand::CopyReports(!copy));
    }
}

fn log_level(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let levels = [
        (LogLevel::Error, i18n::text(&ctx, "settings-log-error")),
        (LogLevel::Warn, i18n::text(&ctx, "settings-log-warn")),
        (LogLevel::Info, i18n::text(&ctx, "settings-log-info")),
        (LogLevel::Debug, i18n::text(&ctx, "settings-log-debug")),
        (LogLevel::Trace, i18n::text(&ctx, "settings-log-trace")),
    ];

    let names: Vec<&str> = levels.iter().map(|(_, name)| &**name).collect();
    let current = levels
        .iter()
        .position(|(level, _)| *level == view.settings.general.log_level);

    let words = texts(&ctx, "settings-log-level");
    row(ui, false, &words.borrowed(), NARROW_CONTROL, |ui| {
        let picked = Select::new(&words.title, &names)
            .selected(current)
            .width(NARROW_CONTROL)
            .show(ui);

        if let Some((level, _)) = picked.value.and_then(|index| levels.get(index)) {
            send(SettingsCommand::LogLevel(*level));
        }
    });
}

fn texts(ctx: &Context, id: &str) -> Texts {
    Texts {
        title: i18n::text(ctx, id),
        description: i18n::text(ctx, &format!("{id}-description")),
    }
}

fn choice<T>(
    ui: &mut Ui,
    words: &Texts,
    options: &[(T, Cow<'static, str>)],
    current: T,
    picked: impl FnOnce(T),
) where
    T: PartialEq + Copy,
{
    let labels: Vec<(T, &str)> = options
        .iter()
        .map(|(value, name)| (*value, &**name))
        .collect();

    let names: Vec<&str> = labels.iter().map(|(_, name)| *name).collect();
    let width = segmented_width(ui, &names);
    row(ui, false, &words.borrowed(), width, |ui| {
        let chosen = segmented(ui, &words.title, &words.title, &labels, current).value;
        if let Some(value) = chosen.filter(|value| *value != current) {
            picked(value);
        }
    });
}
