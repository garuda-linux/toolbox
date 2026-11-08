//! The colour scheme pickers beside Theme: one for dark, one for light,
//! each with System first, which keeps the desktop's colours.

use egui::Ui;
use toolbox_catalog::{ColourSchemeCatalog, SchemeMode};
use toolbox_config::Colours;
use toolbox_gui_kit::widgets::Select;
use toolbox_plan::SchemeId;

use super::{NARROW_CONTROL, Send, texts};
use crate::i18n;
use crate::pages::setting_row::row;
use crate::settings::SettingsCommand;

pub(super) fn scheme(
    ui: &mut Ui,
    mode: SchemeMode,
    (catalog, colours): (&ColourSchemeCatalog, &Colours),
    send: Send<'_>,
) {
    let ctx = ui.ctx().clone();
    let (id, current) = match mode {
        SchemeMode::Dark => ("settings-dark-scheme", colours.dark_scheme.as_ref()),
        SchemeMode::Light => ("settings-light-scheme", colours.light_scheme.as_ref()),
    };

    let ids: Vec<Option<&SchemeId>> = std::iter::once(None)
        .chain(catalog.of(mode).map(|scheme| Some(scheme.id())))
        .collect();

    let system = i18n::text(&ctx, "settings-scheme-system").into_owned();
    let names: Vec<String> = std::iter::once(system)
        .chain(catalog.of(mode).map(|scheme| {
            let text = format!("catalog-colour-schemes-{}", scheme.id());
            i18n::text_or(&ctx, &text, scheme.name()).into_owned()
        }))
        .collect();

    let shown: Vec<&str> = names.iter().map(String::as_str).collect();

    // A scheme the catalog lost shows as System, which the window draws.
    let selected = ids.iter().position(|listed| *listed == current).or(Some(0));

    let words = texts(&ctx, id);
    row(ui, false, &words.borrowed(), NARROW_CONTROL, |ui| {
        let picked = Select::new(&words.title, &shown)
            .selected(selected)
            .width(NARROW_CONTROL)
            .show(ui);

        if let Some(&picked) = picked.and_then(|index| ids.get(index))
            && picked != current
        {
            send(SettingsCommand::Scheme(mode, picked.cloned()));
        }
    });
}
