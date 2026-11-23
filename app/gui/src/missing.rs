//! The window of a build whose catalog folder is missing. Every page
//! would come up empty or broken, so one error state says why, with a way
//! out.

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use eframe::CreationContext;
use egui::{CentralPanel, Context, FontFamily, Id, Rect, Ui, UiBuilder, ViewportCommand, vec2};
use fluent_templates::LanguageIdentifier;
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::toasts::{ToastTexts, Toasts};
use toolbox_gui_kit::widgets::{
    ButtonKind, PropertyTexts, empty_state, pill_button, property_list, rich,
};
use toolbox_gui_kit::{icons, tokens};

use crate::i18n;
use crate::i18n::Arg;

/// The window's size, which fits the state with room around it.
pub const WINDOW: [f32; 2] = [720.0, 420.0];
/// A technical value someone pastes into a terminal, so it is not
/// translated.
const REINSTALL: &str = "sudo pacman -S garuda-toolbox";
/// The state's column, so its lines do not run across a wide window.
const COLUMN: f32 = 520.0;

pub struct MissingCatalog {
    dir: PathBuf,
    toasts: Toasts,
}

impl MissingCatalog {
    fn new(cc: &CreationContext<'_>, language: &LanguageIdentifier, dir: PathBuf) -> Self {
        toolbox_gui_kit::install(&cc.egui_ctx);
        i18n::set_language(&cc.egui_ctx, language);
        Self {
            dir,
            toasts: Toasts::default(),
        }
    }
}

impl eframe::App for MissingCatalog {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let page = CentralPanel::default()
            .show(ui, |ui| show(ui, &self.dir))
            .response
            .rect;

        let ctx = ui.ctx().clone();
        let texts = ToastTexts {
            name: &i18n::text(&ctx, "toast-name"),
        };

        self.toasts.show(&ctx, page, texts);
    }
}

/// # Errors
///
/// When the window cannot open.
pub fn run(app_id: &str, dir: PathBuf) -> eframe::Result {
    let language = i18n::language_from_env(|key| std::env::var_os(key));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(app_id)
            .with_inner_size(WINDOW),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Garuda Toolbox",
        options,
        Box::new(move |cc| Ok(Box::new(MissingCatalog::new(cc, &language, dir)))),
    )
}

/// The error state in the middle of the window, the missing folder and the
/// command that brings it back, each with a copy button, and Quit, which
/// closes the window.
pub fn show(ui: &mut Ui, dir: &Path) {
    let ctx: Context = ui.ctx().clone();

    // egui lays out top down, so the state stands as far down as half the
    // room its last frame left.
    let id = Id::unique("toolbox_gui::missing::height");
    let height = ui.data(|data| data.get_temp::<f32>(id)).unwrap_or(0.0);
    let width = ui.available_width().min(COLUMN);
    let room = ui.available_rect_before_wrap();
    let top = room.top() + ((room.height() - height) / 2.0).max(0.0);
    let left = room.center().x - width / 2.0;
    let column = Rect::from_min_size(egui::pos2(left, top), vec2(width, room.bottom() - top));

    let drawn = ui
        .scope_builder(UiBuilder::new().max_rect(column), |ui| state(ui, dir))
        .response
        .rect
        .height();

    if (drawn - height).abs() > 0.5 {
        ui.data_mut(|data| data.insert_temp(id, drawn));
        ctx.request_discard("the missing-catalog state changed its height");
    }
}

fn state(ui: &mut Ui, dir: &Path) {
    let ctx: Context = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id);
    let gap = tokens::current(&ctx).sizes.card_gap;
    _ = empty_state(
        ui,
        icons::ERROR,
        &text("missing-catalog-title"),
        &text("missing-catalog-body"),
    );

    ui.add_space(gap);
    let copy = |label: &str| {
        i18n::text_with(&ctx, "missing-catalog-copy", &[("label", Arg::from(label))]).into_owned()
    };

    let copied = text("missing-catalog-copied");
    let texts = PropertyTexts {
        copy: &copy,
        copied: &copied,
    };

    let (folder, reinstall) = (
        text("missing-catalog-folder"),
        text("missing-catalog-reinstall"),
    );

    let shown = dir.display().to_string();
    property_list(ui, texts, |list| {
        list.row_with(&folder, Some(shown.as_str()), |row| mono(row, &shown));
        list.row_with(&reinstall, Some(REINSTALL), |row| mono(row, REINSTALL));
    });

    ui.add_space(gap);
    let quit = text("missing-catalog-quit");
    if ui
        .vertical_centered(|ui| pill_button(ui, &quit, ButtonKind::Soft).clicked())
        .inner
    {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

fn mono(ui: &mut Ui, value: &str) {
    let ink = tokens::current(ui.ctx()).text;
    let words = rich(ui, value, Weight::Regular, BODY, ink).family(FontFamily::Monospace);
    ui.add(egui::Label::new(words).wrap());
}
