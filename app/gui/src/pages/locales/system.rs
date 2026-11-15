//! The system language row: a dropdown of the enabled locales, named by
//! language and territory.

use egui::Ui;
use toolbox_core::Snapshot;
use toolbox_core::features::locales::view::{
    system_choice_names, system_choices, system_queued, system_shown,
};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{Select, card};
use toolbox_plan::LocaleName;

use crate::command::LocaleCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::Look;
use crate::pages::setting_row::{Words, row, wide_control};

/// The choices of one system and queue, with their names measured once.
pub type Choices = RowCache<(Same<Snapshot>, u64, Look), Choice>;

#[derive(Debug)]
pub struct Choice {
    codes: Vec<String>,
    names: Vec<String>,
    shown: String,
    /// The widest name, "Not set" and the shown code.
    fitting: f32,
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, cache: &mut Choices) {
    let ctx = ui.ctx().clone();
    let (snapshot, changes) = (context.snapshot, context.changes);
    let unset = i18n::text(&ctx, "locales-system-unset");
    let rest = (Same::of(snapshot), context.revision, Look::of(ui));
    let built = cache.get(&[], rest, || {
        let codes = system_choices(snapshot, changes);
        let names: Vec<String> = codes
            .iter()
            .zip(system_choice_names(snapshot, changes))
            .map(|(code, (language, territory))| choice(&ctx, code, &language, &territory))
            .collect();

        let shown = system_shown(snapshot, changes);
        let measured = names.iter().map(String::as_str).chain([&*unset, &*shown]);
        let fitting = Select::fitting_width(ui, measured);

        Choice {
            codes,
            names,
            shown,
            fitting,
        }
    });

    let title = i18n::text(&ctx, "locales-system");
    let description = i18n::text(&ctx, "locales-system-detail");
    let queued = (!system_queued(changes).is_empty()).then(|| i18n::text(&ctx, "locales-queued"));
    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: None,
        queued: queued.as_deref(),
    };

    // The whole locale code at a name's end outweighs the 40 % cap.
    let width = built.fitting.max(wide_control(ui));
    let shown = &built.shown;

    card(ui, |ui| {
        row(ui, true, &words, width, |ui| {
            let options: Vec<&str> = built.names.iter().map(String::as_str).collect();
            let current = built.codes.iter().position(|code| code == shown);
            let placeholder = if shown.is_empty() { &*unset } else { shown };
            let picked = ui
                .enabled_ui(!built.codes.is_empty() && !context.busy, |ui| {
                    Select::new(&title, &options)
                        .selected(current)
                        .placeholder(placeholder)
                        .width(width.min(ui.available_width()))
                        .show(ui)
                })
                .inner;

            let locale = picked
                .value
                .and_then(|index| built.codes.get(index))
                .and_then(|code| LocaleName::new(code.clone()).ok());

            if let Some(locale) = locale {
                context.commands.send(LocaleCommand::System(locale));
            }
        });
    });
}

pub fn choice(ctx: &egui::Context, code: &str, language: &str, territory: &str) -> String {
    if language.is_empty() {
        return code.to_owned();
    }

    let mut args = vec![("language", language)];
    let id = if territory.is_empty() {
        "locales-system-choice"
    } else {
        args.push(("territory", territory));
        "locales-system-choice-territory"
    };

    args.push(("code", code));
    i18n::text_with(ctx, id, &args).into_owned()
}
