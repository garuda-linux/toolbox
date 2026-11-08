use egui::Ui;
use toolbox_core::features::bootrepair::page::{self, TargetChoice};
use toolbox_core::features::bootrepair::{Encryption, Firmware, RepairFacts, System};
use toolbox_gui_kit::widgets::{
    ButtonKind, Select, Tone, banner, card, pill_button, section_intro,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::RootLayout;

use super::dialog::Dialog;
use super::{BootRepairPage, texts};
use crate::i18n;
use crate::pages::setting_row::{Words, row};
use crate::pages::{PageContext, heading};

const WIDE_CONTROL: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_wide;
const LAYOUTS: [RootLayout; 2] = [RootLayout::Subvolume, RootLayout::Plain];

/// Returns the dialog a repair button asks for.
pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (facts, system): (&RepairFacts, &System),
    page: &mut BootRepairPage,
) -> Option<Dialog> {
    installed(ui, (facts, system), page);
    ui.add_space(ui.tokens().sizes.section_gap);
    repair(ui, context, (facts, system), page)
}

const fn words<'a>(title: &'a str, description: &'a str) -> Words<'a> {
    Words {
        current: None,
        title,
        description,
        note: None,
        queued: None,
    }
}

fn installed(ui: &mut Ui, (facts, system): (&RepairFacts, &System), page: &mut BootRepairPage) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    heading(ui, icons::COMPUTER, "repair-system");
    card(ui, |ui| {
        root(ui, (facts, system), page);
        let (firmware, firmware_detail) = texts::firmware(&ctx, facts.firmware);
        fact(ui, (&text("repair-firmware"), &firmware_detail), &firmware);
        let locked =
            (system.encryption == Encryption::Locked).then(|| text("repair-locked-detail"));

        fact(
            ui,
            (
                &text("repair-encryption"),
                locked.as_deref().unwrap_or_default(),
            ),
            &texts::encryption(&ctx, system.encryption),
        );

        fact(ui, (&text("repair-disk"), ""), &texts::disks(system));
        if let Some(chosen) = page::layout(system, page.layout) {
            layout(ui, chosen, page);
        }
    });

    if system.in_use {
        ui.add_space(ui.tokens().sizes.card_gap);
        banner(ui, Tone::Warning, &text("repair-in-use"), &[]);
    }
}

fn fact(ui: &mut Ui, (title, detail): (&str, &str), value: &str) {
    row(ui, false, &words(title, detail), WIDE_CONTROL, |ui| {
        ui.label(value);
    });
}

fn layout(ui: &mut Ui, chosen: RootLayout, page: &mut BootRepairPage) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "repair-layout").into_owned();
    let detail = i18n::text(&ctx, "repair-layout-detail").into_owned();
    let names: Vec<String> = LAYOUTS
        .iter()
        .map(|each| texts::layout_short(&ctx, *each))
        .collect();

    let options: Vec<&str> = names.iter().map(String::as_str).collect();
    let current = LAYOUTS.iter().position(|each| *each == chosen);
    row(ui, false, &words(&title, &detail), WIDE_CONTROL, |ui| {
        if let Some(picked) = Select::new(&title, &options)
            .selected(current)
            .width(WIDE_CONTROL)
            .show(ui)
            .and_then(|index| LAYOUTS.get(index))
        {
            page.layout = Some(*picked);
        }
    });
}

fn repair(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (facts, system): (&RepairFacts, &System),
    page: &mut BootRepairPage,
) -> Option<Dialog> {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let title = text("repair-work");
    section_intro(
        ui,
        icons::RUN_BUILD_CONFIGURE,
        &title,
        &text("repair-asks-first"),
    );

    let choices = page::targets(facts);
    let target = page::target(facts, page.target.as_ref(), Some(system));
    if choices.is_empty() {
        let id = match facts.firmware {
            Firmware::Uefi => "repair-no-esp",
            Firmware::Bios => "repair-no-disk",
        };

        banner(ui, Tone::Warning, &text(id), &[]);
    } else {
        card(ui, |ui| {
            targets(ui, facts.firmware, &choices, target.as_ref(), page);
        });
    }

    ui.add_space(ui.tokens().sizes.card_gap);

    let free = !system.in_use && !context.busy;
    let mut dialog = None;
    ui.horizontal(|ui| {
        let reinstall = ui
            .add_enabled_ui(free && target.is_some(), |ui| {
                pill_button(ui, &text("repair-reinstall"), ButtonKind::Primary)
            })
            .inner;

        if reinstall.clicked() {
            dialog = Some(Dialog::new(
                system,
                target.map(|choice| choice.target),
                page.layout,
            ));
        }

        let regenerate = ui
            .add_enabled_ui(free, |ui| {
                pill_button(ui, &text("repair-regenerate"), ButtonKind::Soft)
            })
            .inner;

        if regenerate.clicked() {
            dialog = Some(Dialog::new(system, None, page.layout));
        }
    });

    dialog
}

fn targets(
    ui: &mut Ui,
    firmware: Firmware,
    choices: &[TargetChoice],
    current: Option<&TargetChoice>,
    page: &mut BootRepairPage,
) {
    let ctx = ui.ctx().clone();
    let (title, detail) = match firmware {
        Firmware::Uefi => ("repair-esp", "repair-esp-detail"),
        Firmware::Bios => ("repair-target-disk", "repair-target-disk-detail"),
    };

    let title = i18n::text(&ctx, title).into_owned();
    let detail = i18n::text(&ctx, detail).into_owned();
    let lines: Vec<String> = choices.iter().map(texts::target_line).collect();
    let names: Vec<&str> = lines.iter().map(String::as_str).collect();
    let selected = current.and_then(|current| choices.iter().position(|each| each == current));
    row(ui, true, &words(&title, &detail), WIDE_CONTROL, |ui| {
        if let Some(picked) = Select::new(&title, &names)
            .selected(selected)
            .width(WIDE_CONTROL)
            .show(ui)
            .and_then(|index| choices.get(index))
        {
            page.target = Some(picked.target.clone());
        }
    });
}

/// Another system has its own usual target and layout.
fn root(ui: &mut Ui, (facts, system): (&RepairFacts, &System), page: &mut BootRepairPage) {
    let ctx = ui.ctx().clone();
    let systems = &facts.inventory.systems;
    let lines: Vec<String> = systems.iter().map(texts::system_line).collect();
    let names: Vec<&str> = lines.iter().map(String::as_str).collect();
    let current = systems
        .iter()
        .position(|each| each.device() == system.device());

    let title = i18n::text(&ctx, "repair-root").into_owned();
    let detail = i18n::text(&ctx, "repair-root-detail").into_owned();
    row(ui, true, &words(&title, &detail), WIDE_CONTROL, |ui| {
        if let Some(index) = Select::new(&title, &names)
            .selected(current)
            .width(WIDE_CONTROL)
            .show(ui)
            && let Some(picked) = systems.get(index)
        {
            page.system = Some(picked.device().to_owned());
            page.target = None;
            page.layout = None;
        }
    });
}
