//! The system language row: a dropdown of the enabled locales, named by
//! language and territory.

use egui::Ui;
use toolbox_core::features::locales::view::{
    system_choice_names, system_choices, system_queued, system_shown,
};
use toolbox_core::{ChangeSet, Snapshot};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::row_cache::{RowCache, Same};
use toolbox_gui_kit::widgets::{Select, card};
use toolbox_plan::LocaleName;

use crate::command::LocaleCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::rows::Look;
use crate::pages::setting_row::{Words, row, wide_control};

/// The choices of one system and queue, with their names measured once.
pub type Choices = RowCache<(Same<Snapshot>, u64, Look), Choice>;

#[derive(Debug)]
pub struct Choice {
    codes: Vec<String>,
    names: Vec<String>,
    shown: Option<String>,
    /// The widest name, "Not set" and the shown code.
    fitting: f32,
}

fn build(
    ui: &Ui,
    snapshot: &Snapshot,
    changes: &ChangeSet,
    ctx: &egui::Context,
    unset: &str,
) -> Choice {
    let codes = system_choices(snapshot, changes);
    let names: Vec<String> = codes
        .iter()
        .zip(system_choice_names(snapshot, changes))
        .map(|(code, (language, territory))| {
            choice(ctx, code, language.as_deref(), territory.as_deref())
        })
        .collect();

    let shown = system_shown(snapshot, changes).map(|name| name.as_str().to_owned());
    let measured = names
        .iter()
        .map(String::as_str)
        .chain([unset])
        .chain(shown.as_deref());

    let fitting = Select::fitting_width(ui, measured);

    Choice {
        codes,
        names,
        shown,
        fitting,
    }
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, cache: &mut Choices) {
    let ctx = ui.ctx().clone();
    let (snapshot, changes) = (context.snapshot, context.changes);
    let unset = i18n::text(&ctx, "locales-system-unset");
    let rest = (Same::of(snapshot), context.revision, Look::of(ui));
    let built = cache.get(&[], rest, || build(ui, snapshot, changes, &ctx, &unset));

    let title = i18n::text(&ctx, "locales-system");
    let description = i18n::text(&ctx, "locales-system-detail");
    let queued = system_queued(changes)
        .is_some()
        .then(|| i18n::text(&ctx, "locales-queued"));

    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: None,
        queued: queued.as_deref(),
    };

    // The whole locale code at a name's end outweighs the 40 % cap.
    let width = built.fitting.max(wide_control(ui));
    let shown = built.shown.as_deref();

    card(ui, |ui| {
        row(ui, true, &words, width, |ui| {
            let options: Vec<&str> = built.names.iter().map(String::as_str).collect();
            let current = built
                .codes
                .iter()
                .position(|code| Some(code.as_str()) == shown);

            let placeholder = shown.unwrap_or(&unset);
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

pub fn choice(
    ctx: &egui::Context,
    code: &str,
    language: Option<&str>,
    territory: Option<&str>,
) -> String {
    let Some(language) = language else {
        return code.to_owned();
    };

    let mut args = vec![("language", Arg::from(language))];
    let id = territory.map_or("locales-system-choice", |territory| {
        args.push(("territory", Arg::from(territory)));
        "locales-system-choice-territory"
    });

    args.push(("code", Arg::from(code)));
    i18n::text_with(ctx, id, &args).into_owned()
}
