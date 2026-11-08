mod minutes;
mod schemes;
#[cfg(test)]
mod tests;

use std::borrow::Cow;

use egui::{Context, Label, Ui};
use toolbox_catalog::SchemeMode;
use toolbox_config::{LogLevel, RefreshInterval};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::widgets::{SectionHeading, Select, card, rich, segmented, segmented_width};

use self::minutes::MinutesField;
use super::header::title_header;
use super::setting_row::{Words, row, switch_row};
use super::{PageContext, PageView};
use crate::i18n;
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
            let schemes = (&context.catalogs.colour_schemes, &view.settings.colours);
            schemes::scheme(ui, SchemeMode::Dark, schemes, &send);
            schemes::scheme(ui, SchemeMode::Light, schemes, &send);
            motion(ui, &view, &send);
        });
        ui.add_space(ui.tokens().sizes.section_gap);

        section(ui, "settings-behaviour", icons::CHRONOMETER);
        card(ui, |ui| {
            autostart(ui, &view, &send);
            only_when_asked(ui, &view, &send);
            self.refresh(ui, &view, &send);
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
    fn refresh(&mut self, ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
        let ctx = ui.ctx().clone();
        let refresh = view.settings.status.refresh;
        let held = self.minutes.held().map(|kept| {
            i18n::text_with(
                &ctx,
                "settings-refresh-clamped",
                &[("minutes", &kept.to_string())],
            )
        });

        let off =
            (refresh == RefreshInterval::Off).then(|| i18n::text(&ctx, "settings-refresh-off"));

        let words = Words {
            current: None,
            title: &i18n::text(&ctx, "settings-refresh"),
            description: &i18n::text(&ctx, "settings-refresh-description"),
            note: held.as_deref().or(off.as_deref()),
            queued: None,
        };

        let name = i18n::text(&ctx, "settings-refresh-minutes");
        let unit = i18n::text(&ctx, "settings-refresh-unit");
        let field = &mut self.minutes;
        row(ui, false, &words, MINUTES_WIDTH, |ui| {
            let enabled = refresh != RefreshInterval::Off;
            let committed = ui
                .add_enabled_ui(enabled, |ui| {
                    let texts = (name.as_ref(), unit.as_ref());
                    field.show(ui, minutes_of(refresh), texts, MINUTES_WIDTH)
                })
                .inner;

            if let Some(interval) = committed {
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
    let tokens = ui.tokens();
    let help = i18n::text(&ctx, &format!("{id}-help"));
    SectionHeading::new(icon, &i18n::text(&ctx, id), "").show(ui);
    ui.add_space(tokens.sizes.tight_gap);
    ui.add(Label::new(rich(ui, &help, Weight::Regular, SMALL, tokens.secondary)).wrap());
    ui.add_space(tokens.sizes.heading_gap);
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

        if let Some(code) = picked.and_then(|index| codes.get(index)) {
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

/// Checks only when the user asks. Turned back on, the last minutes stay.
fn only_when_asked(ui: &mut Ui, view: &SettingsView<'_>, send: Send<'_>) {
    let ctx = ui.ctx().clone();
    let refresh = view.settings.status.refresh;
    let words = texts(&ctx, "settings-refresh-ask");

    let asked = refresh == RefreshInterval::Off;
    if switch_row(ui, false, &words.borrowed(), asked, true) {
        let next = if asked {
            RefreshInterval::Minutes(minutes_of(refresh))
        } else {
            RefreshInterval::Off
        };

        send(SettingsCommand::Refresh(next));
    }
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

        if let Some((level, _)) = picked.and_then(|index| levels.get(index)) {
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
        let chosen = segmented(ui, &words.title, &labels, current);
        if let Some(value) = chosen.filter(|value| *value != current) {
            picked(value);
        }
    });
}

/// The minutes the field shows: the setting's, or the usual hour while
/// the checks wait to be asked for.
const fn minutes_of(refresh: RefreshInterval) -> u16 {
    match refresh {
        RefreshInterval::Minutes(minutes) => minutes,
        RefreshInterval::Off => toolbox_config::DEFAULT_MINUTES,
    }
}
