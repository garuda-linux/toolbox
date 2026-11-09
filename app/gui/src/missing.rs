//! The window of a build whose catalog folder is missing. Every page
//! would come up empty or broken, so one banner says why, with a way out.

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use eframe::CreationContext;
use egui::{CentralPanel, Context, Ui, ViewportCommand};
use fluent_templates::LanguageIdentifier;
use toolbox_gui_kit::widgets::{Tone, banner};

use crate::i18n;

pub struct MissingCatalog {
    dir: PathBuf,
}

impl MissingCatalog {
    fn new(cc: &CreationContext<'_>, language: &LanguageIdentifier, dir: PathBuf) -> Self {
        toolbox_gui_kit::install(&cc.egui_ctx);
        i18n::set_language(&cc.egui_ctx, language);
        Self { dir }
    }
}

impl eframe::App for MissingCatalog {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ui, |ui| show(ui, &self.dir));
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
            .with_inner_size([720.0, 240.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Garuda Toolbox",
        options,
        Box::new(move |cc| Ok(Box::new(MissingCatalog::new(cc, &language, dir)))),
    )
}

/// The banner and its Quit, which closes the window.
pub fn show(ui: &mut Ui, dir: &Path) {
    let ctx: Context = ui.ctx().clone();
    let shown = dir.display().to_string();
    let text = i18n::text_with(&ctx, "missing-catalog", &[("dir", &shown)]);
    let quit = i18n::text(&ctx, "missing-catalog-quit");

    if banner(ui, Tone::Error, &text, &[&quit]) == Some(0) {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}
