//! The appearance card: Plymouth's theme while the splash is on, and the
//! picture and theme of the GRUB menu, each picked in the desktop's file
//! dialog.

use egui::Ui;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::boot::view::{self, Page};
use toolbox_core::features::boot::{Flag, plymouth_queued};
use toolbox_gui_kit::widgets::{
    ButtonKind, Select, Tone, banner, card, icon_button, icon_button_width, pill_button, pill_width,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::GrubKey;

use super::{heading, queued, send};
use crate::boot::{Boot, FileKind};
use crate::command::BootCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::setting_row::{Words, row, wide_control};

struct FileField<'a> {
    kind: FileKind,
    id: &'static str,
    path: &'a str,
    queued: bool,
    /// The path came from elsewhere, so there is nothing to clear.
    note: Option<String>,
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
    let ctx = ui.ctx().clone();
    heading(ui, "boot-look");
    let splash = page.flag(Flag::Splash);
    let background = page.background();
    let theme = page.theme();
    let from_theme = page.background_from_theme();

    // Beside the rows whose Choose did nothing, since a cancelled dialog
    // and one that never opened look the same otherwise.
    if context.service::<Boot>().chooser_failed() {
        let text = i18n::text(&ctx, "boot-file-chooser-failed");
        banner(ui, Tone::Warning, &text, &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }

    card(ui, |ui| {
        if splash {
            plymouth(ui, context);
        }

        file(
            ui,
            context,
            &FileField {
                kind: FileKind::Background,
                id: "boot-background",
                path: background,
                queued: view::is_queued(context.changes, GrubKey::Background),
                note: from_theme
                    .then(|| i18n::text(&ctx, "boot-background-from-theme").into_owned()),
            },
            !splash,
        );

        file(
            ui,
            context,
            &FileField {
                kind: FileKind::Theme,
                id: "boot-theme",
                path: theme,
                queued: view::is_queued(context.changes, GrubKey::Theme),
                note: None,
            },
            false,
        );
    });
}

/// systemd-boot has no menu picture or theme.
pub fn plymouth_card(ui: &mut Ui, context: &PageContext<'_>) {
    heading(ui, "boot-look");
    card(ui, |ui| plymouth(ui, context));
}

fn plymouth(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let (snapshot, changes) = (context.snapshot, context.changes);
    let installed = view::plymouth_installed(snapshot);
    let description = if installed {
        "boot-plymouth-detail"
    } else if plymouth_queued(changes) {
        "boot-plymouth-queued"
    } else {
        "boot-plymouth-missing"
    };

    let title = i18n::text(&ctx, "boot-plymouth");
    let description = i18n::text(&ctx, description);
    let tag = queued(&ctx, !view::plymouth_theme_queued(changes).is_empty());
    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: None,
        queued: tag.as_deref(),
    };

    let width = wide_control(ui);
    row(ui, true, &words, width, |ui| {
        let themes = view::plymouth_themes(snapshot);
        let names: Vec<&str> = themes.iter().map(String::as_str).collect();
        let shown = view::plymouth_shown(snapshot, changes);
        let current = themes.iter().position(|theme| *theme == shown);
        let none = i18n::text(&ctx, "boot-plymouth-none");
        let picked = ui
            .enabled_ui(installed && !themes.is_empty(), |ui| {
                Select::new(&title, &names)
                    .selected(current)
                    .placeholder(&none)
                    .width(width)
                    .show(ui)
            })
            .inner;

        if let Some(theme) = picked.value.and_then(|index| themes.get(index)) {
            send(context, Edit::PlymouthTheme(theme.clone()));
        }
    });
}

fn file(ui: &mut Ui, context: &PageContext<'_>, field: &FileField<'_>, first: bool) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, field.id);
    let description = i18n::text(&ctx, &format!("{}-detail", field.id));
    let tag = queued(&ctx, field.queued);

    // The file chosen now reads as part of the row's words, so a long path
    // is cut there instead of widening the controls.
    let none_word = i18n::text(&ctx, "boot-file-none");
    let current = if field.path.is_empty() {
        &*none_word
    } else {
        field.path
    };

    let words = Words {
        current: Some(current),
        title: &title,
        description: &description,
        note: field.note.as_deref(),
        queued: tag.as_deref(),
    };

    let choose = i18n::text(&ctx, "boot-file-choose");
    let gap = ui.spacing().item_spacing.x;
    let width = pill_width(ui, &choose) + gap + icon_button_width();
    row(ui, first, &words, width, |ui| {
        let choose_name = i18n::text_with(
            &ctx,
            "boot-file-choose-name",
            &[("name", Arg::from(&title))],
        );

        let chosen = pill_button(ui, &choose, ButtonKind::Soft).accessible_name(choose_name);

        if chosen.clicked() {
            context.commands.send(BootCommand::Choose {
                kind: field.kind,
                title: i18n::text(&ctx, &format!("{}-title", field.id)).into_owned(),
                filter: i18n::text(&ctx, &format!("{}-filter", field.id)).into_owned(),
            });
        }

        // It stays in place while there is nothing to clear, so the
        // Choose buttons of both rows keep one edge.
        let clearable = !field.path.is_empty() && field.note.is_none();
        let clear = i18n::text_with(&ctx, "boot-file-clear", &[("name", Arg::from(&title))]);
        let cleared = ui
            .enabled_ui(clearable, |ui| icon_button(ui, icons::WINDOW_CLOSE, &clear))
            .inner
            .clicked();

        if cleared {
            let none = String::new();
            let change = match field.kind {
                FileKind::Background => Edit::Background(none),
                FileKind::Theme => Edit::Theme(none),
            };

            send(context, change);
        }
    });
}
