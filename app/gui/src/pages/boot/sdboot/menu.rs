use egui::Ui;
use toolbox_core::domains::LoaderSettingsDomain;
use toolbox_gui_kit::widgets::{Select, number_field_with_unit};
use toolbox_plan::LoaderDefault;
use toolbox_plan::sdboot::{LoaderKey, LoaderSetting, LoaderSettings, LoaderTimeout, SdBootMenu};

use super::super::menu::{TIMEOUT_FIELD_WIDTH, TIMEOUT_MAX, step};
use super::super::{WIDE_CONTROL, queued};
use crate::command::BootCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::setting_row::{Words, row, switch_row};

const FIRST_SECONDS: u32 = 5;

const TIMEOUT_CONTROL_WIDTH: f32 = 400.0;

const TIMEOUT_SELECT_WIDTH: f32 = 160.0;

pub fn show(ui: &mut Ui, context: &PageContext<'_>, menu: &SdBootMenu) {
    let settings = wanted(&menu.settings, context);

    default_entry(ui, context, menu, &settings);
    remember(ui, context, &settings);
    timeout(ui, context, &settings);
    switch(ui, context, LoaderKey::Editor, settings.editor);
    switch(ui, context, LoaderKey::AutoEntries, settings.auto_entries);
    switch(ui, context, LoaderKey::AutoFirmware, settings.auto_firmware);
}

fn wanted(file: &LoaderSettings, context: &PageContext<'_>) -> LoaderSettings {
    let mut settings = file.clone();
    let queued = |key| LoaderSettingsDomain::queued(context.changes, key);

    for key in [
        LoaderKey::Timeout,
        LoaderKey::Default,
        LoaderKey::Editor,
        LoaderKey::AutoEntries,
        LoaderKey::AutoFirmware,
    ] {
        match queued(key) {
            Some(LoaderSetting::Timeout(timeout)) => settings.timeout = timeout,
            Some(LoaderSetting::Default(default)) => settings.default = default,
            Some(LoaderSetting::Editor(on)) => settings.editor = on,
            Some(LoaderSetting::AutoEntries(on)) => settings.auto_entries = on,
            Some(LoaderSetting::AutoFirmware(on)) => settings.auto_firmware = on,
            None => {}
        }
    }

    settings
}

fn tag(context: &PageContext<'_>, ctx: &egui::Context, key: LoaderKey) -> Option<String> {
    queued(
        ctx,
        LoaderSettingsDomain::queued(context.changes, key).is_some(),
    )
}

fn send(context: &PageContext<'_>, setting: LoaderSetting) {
    context.commands.send(BootCommand::Loader(setting));
}

fn default_entry(
    ui: &mut Ui,
    context: &PageContext<'_>,
    menu: &SdBootMenu,
    settings: &LoaderSettings,
) {
    let ctx = ui.ctx().clone();
    let entries: Vec<_> = menu.choosable().collect();
    let names: Vec<&str> = entries.iter().map(|entry| entry.title.as_str()).collect();
    let remembered = settings.default.as_ref() == Some(&LoaderDefault::saved());
    let current = settings.default.as_ref().and_then(|default| {
        entries.iter().position(|entry| {
            entry.default_pattern() == *default || entry.id.as_str() == default.as_str()
        })
    });

    let placeholder = match &settings.default {
        None => i18n::text(&ctx, "boot-sdboot-default-first").into_owned(),
        Some(_) if remembered => i18n::text(&ctx, "boot-default-last").into_owned(),
        Some(other) => other.as_str().to_owned(),
    };

    let title = i18n::text(&ctx, "boot-default");
    let description = i18n::text(&ctx, "boot-default-known");
    let tag = tag(context, &ctx, LoaderKey::Default);
    let words = Words {
        title: &title,
        description: &description,
        note: None,
        queued: tag.as_deref(),
        current: None,
    };

    row(ui, true, &words, WIDE_CONTROL, |ui| {
        let picked = ui
            .add_enabled_ui(!remembered && !entries.is_empty(), |ui| {
                Select::new(&title, &names)
                    .selected(current)
                    .placeholder(&placeholder)
                    .width(WIDE_CONTROL)
                    .show(ui)
            })
            .inner;

        if let Some(entry) = picked.and_then(|index| entries.get(index)) {
            send(
                context,
                LoaderSetting::Default(Some(entry.default_pattern())),
            );
        }
    });
}

/// Off cannot know the default from before `@saved`, so it goes back to
/// the first entry.
fn remember(ui: &mut Ui, context: &PageContext<'_>, settings: &LoaderSettings) {
    let ctx = ui.ctx().clone();
    let on = settings.default.as_ref() == Some(&LoaderDefault::saved());
    let title = i18n::text(&ctx, "boot-remember");
    let description = i18n::text(&ctx, "boot-remember-detail");
    let tag = tag(context, &ctx, LoaderKey::Default);
    let words = Words {
        title: &title,
        description: &description,
        note: None,
        queued: tag.as_deref(),
        current: None,
    };

    if switch_row(ui, false, &words, on, true) {
        send(
            context,
            LoaderSetting::Default((!on).then(LoaderDefault::saved)),
        );
    }
}

fn timeout(ui: &mut Ui, context: &PageContext<'_>, settings: &LoaderSettings) {
    let ctx = ui.ctx().clone();
    let modes = [
        i18n::text(&ctx, "boot-sdboot-unset").into_owned(),
        i18n::text(&ctx, "boot-sdboot-timeout-seconds").into_owned(),
        i18n::text(&ctx, "boot-sdboot-timeout-menu-force").into_owned(),
        i18n::text(&ctx, "boot-sdboot-timeout-menu-hidden").into_owned(),
    ];

    let names: Vec<&str> = modes.iter().map(String::as_str).collect();
    let mode = match settings.timeout {
        None => Some(0),
        Some(LoaderTimeout::Seconds(_)) => Some(1),
        Some(LoaderTimeout::MenuForce) => Some(2),
        Some(LoaderTimeout::MenuHidden) => Some(3),
        Some(LoaderTimeout::MenuDisabled) => None,
    };

    let title = i18n::text(&ctx, "boot-timeout");
    let description = i18n::text(&ctx, "boot-sdboot-timeout-detail");
    let tag = tag(context, &ctx, LoaderKey::Timeout);
    let words = Words {
        title: &title,
        description: &description,
        note: None,
        queued: tag.as_deref(),
        current: None,
    };

    row(ui, false, &words, TIMEOUT_CONTROL_WIDTH, |ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            // As wide as the other selects, unless the seconds need room.
            let width = if mode == Some(1) {
                TIMEOUT_SELECT_WIDTH
            } else {
                WIDE_CONTROL
            };

            let picked = Select::new(&title, &names)
                .selected(mode)
                .placeholder(&i18n::text(&ctx, "boot-sdboot-timeout-menu-disabled"))
                .width(width)
                .show(ui);

            let wanted = match picked {
                Some(0) => Some(None),
                Some(1) => Some(Some(LoaderTimeout::Seconds(FIRST_SECONDS))),
                Some(2) => Some(Some(LoaderTimeout::MenuForce)),
                Some(3) => Some(Some(LoaderTimeout::MenuHidden)),
                Some(_) | None => seconds(ui, &ctx, settings.timeout).map(Some),
            };

            if let Some(timeout) = wanted.filter(|timeout| *timeout != settings.timeout) {
                send(context, LoaderSetting::Timeout(timeout));
            }
        });
    });
}

fn seconds(
    ui: &mut Ui,
    ctx: &egui::Context,
    timeout: Option<LoaderTimeout>,
) -> Option<LoaderTimeout> {
    let Some(LoaderTimeout::Seconds(now)) = timeout else {
        return None;
    };

    let mut value = now.min(TIMEOUT_MAX);

    if step(ui, "−", &i18n::text(ctx, "boot-timeout-less")) {
        value = value.saturating_sub(1);
    }

    number_field_with_unit(
        ui,
        &mut value,
        0..=TIMEOUT_MAX,
        TIMEOUT_FIELD_WIDTH,
        (
            &i18n::text(ctx, "boot-timeout-seconds"),
            &i18n::text(ctx, "boot-timeout-unit"),
        ),
    );

    if step(ui, "+", &i18n::text(ctx, "boot-timeout-more")) {
        value = value.saturating_add(1);
    }

    Some(LoaderTimeout::Seconds(value.min(TIMEOUT_MAX)))
}

/// Three ways, as systemd-boot's own default differs between versions
/// and the page cannot know it.
fn switch(ui: &mut Ui, context: &PageContext<'_>, key: LoaderKey, value: Option<bool>) {
    let ctx = ui.ctx().clone();
    let id = format!("boot-sdboot-{}", key.as_str());
    let title = i18n::text(&ctx, &id);
    let description = i18n::text(&ctx, &format!("{id}-detail"));
    let choices = [
        i18n::text(&ctx, "boot-sdboot-unset").into_owned(),
        i18n::text(&ctx, "boot-sdboot-on").into_owned(),
        i18n::text(&ctx, "boot-sdboot-off").into_owned(),
    ];

    let names: Vec<&str> = choices.iter().map(String::as_str).collect();
    let tag = tag(context, &ctx, key);
    let words = Words {
        title: &title,
        description: &description,
        note: None,
        queued: tag.as_deref(),
        current: None,
    };

    row(ui, false, &words, WIDE_CONTROL, |ui| {
        let selected = match value {
            None => 0,
            Some(true) => 1,
            Some(false) => 2,
        };

        let picked = Select::new(&title, &names)
            .selected(Some(selected))
            .width(WIDE_CONTROL)
            .show(ui);

        let wanted = match picked {
            Some(1) => Some(Some(true)),
            Some(2) => Some(Some(false)),
            Some(0) => Some(None),
            Some(_) | None => None,
        };

        if let Some(on) = wanted.filter(|on| *on != value) {
            let setting = match key {
                LoaderKey::Editor => LoaderSetting::Editor(on),
                LoaderKey::AutoEntries => LoaderSetting::AutoEntries(on),
                LoaderKey::AutoFirmware => LoaderSetting::AutoFirmware(on),
                LoaderKey::Timeout | LoaderKey::Default => return,
            };

            send(context, setting);
        }
    });
}
