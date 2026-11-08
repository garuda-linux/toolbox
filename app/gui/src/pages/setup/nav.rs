//! Back, the list of every stage and Next, fixed below each stage but the
//! last. On the Summary, Next reviews and applies, or shows the success
//! page when nothing was picked.

use egui::{Align, Layout, Ui};
use toolbox_core::features::toggles::summary::summary;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{ButtonKind, Select, pill_button, rich};

use super::{SetupPage, Stage};
use crate::command::UiCommand;
use crate::i18n;
use crate::pages::PageContext;

const NARROW_CONTROL: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_narrow;

pub fn stages(ctx: &egui::Context, steps: &[(String, String)]) -> Vec<(Stage, String)> {
    let numbered = steps.iter().enumerate().map(|(index, (id, title))| {
        let number = (index + 1).to_string();
        let name = i18n::text_with(
            ctx,
            "setup-step-item",
            &[("number", &number), ("title", title)],
        );

        (Stage::Step(id.clone()), name.into_owned())
    });

    std::iter::once((
        Stage::Welcome,
        i18n::text(ctx, "setup-welcome").into_owned(),
    ))
    .chain(numbered)
    .chain(std::iter::once((
        Stage::Summary,
        i18n::text(ctx, "setup-summary").into_owned(),
    )))
    .collect()
}

pub fn bar(
    ui: &mut Ui,
    context: &PageContext<'_>,
    page: &mut SetupPage,
    stages: &[(Stage, String)],
) -> bool {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let current = stages.iter().position(|(stage, _)| *stage == page.stage);
    let mut moved = false;
    let middle = ui.max_rect().center().x;
    ui.horizontal(|ui| {
        let back = ui
            .add_enabled_ui(current.unwrap_or_default() > 0, |ui| {
                pill_button(ui, &text("setup-back"), ButtonKind::Soft)
            })
            .inner;

        if back.clicked() {
            moved |= go(page, stages, current, false);
        }

        // Next at the right edge, the list centred under the page, so it
        // stays put while Next changes its words.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let next = next_label(context, page);
            if pill_button(ui, &text(next), ButtonKind::Primary).clicked() {
                moved |= forward(context, page, stages, current);
            }

            let width = NARROW_CONTROL.min(ui.available_width());
            let right = ui.cursor().right();
            ui.add_space((right - (middle + width / 2.0)).max(0.0));
            let names: Vec<&str> = stages.iter().map(|(_, name)| name.as_str()).collect();
            if let Some((stage, _)) = Select::new(&text("setup-go-to"), &names)
                .selected(current)
                .width(width)
                .show(ui)
                .and_then(|index| stages.get(index))
            {
                page.stage = stage.clone();
                moved = true;
            }
        });
    });

    let secondary = ui.tokens().secondary;
    ui.vertical_centered(|ui| {
        ui.label(rich(
            ui,
            &text("setup-keys"),
            Weight::Regular,
            SMALL,
            secondary,
        ));
    });
    moved
}

/// Alt+Right and Alt+Left, which the page took from the input.
pub fn keys(
    context: &PageContext<'_>,
    page: &mut SetupPage,
    stages: &[(Stage, String)],
    (next, back): (bool, bool),
) {
    let current = stages.iter().position(|(stage, _)| *stage == page.stage);
    match (next, back) {
        (true, _) => {
            forward(context, page, stages, current);
        }
        (false, true) => {
            go(page, stages, current, false);
        }
        (false, false) => {}
    }
}

fn next_label(context: &PageContext<'_>, page: &SetupPage) -> &'static str {
    match page.stage {
        Stage::Summary if picked(context) => "setup-review",
        Stage::Summary => "setup-finish",
        Stage::Welcome | Stage::Step(_) | Stage::Success => "setup-next",
    }
}

fn forward(
    context: &PageContext<'_>,
    page: &mut SetupPage,
    stages: &[(Stage, String)],
    current: Option<usize>,
) -> bool {
    if page.stage != Stage::Summary {
        return go(page, stages, current, true);
    }

    if picked(context) {
        // A second review after the apply keeps the first mark, or the
        // apply that already ran would never count.
        page.review_at.get_or_insert(context.plans.finished);
        context.commands.send(UiCommand::ReviewChanges);
    } else {
        page.stage = Stage::Success;
    }

    true
}

fn go(
    page: &mut SetupPage,
    stages: &[(Stage, String)],
    current: Option<usize>,
    ahead: bool,
) -> bool {
    let at = current.unwrap_or_default();
    let target = if ahead {
        at.checked_add(1)
    } else {
        at.checked_sub(1)
    };

    let Some((stage, _)) = target.and_then(|index| stages.get(index)) else {
        return false;
    };

    page.stage = stage.clone();
    true
}

fn picked(context: &PageContext<'_>) -> bool {
    context
        .catalogs
        .setup
        .catalog
        .as_ref()
        .is_ok_and(|catalog| !summary(catalog, context.snapshot, context.changes).is_empty())
}
