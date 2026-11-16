//! Where the assistant stands, then Back, the list of every stage and
//! Next, fixed below each stage but the last. On the Summary, Next reviews
//! and applies, or shows the success page when nothing was picked.

#[cfg(test)]
mod tests;

use egui::accesskit::Role;
use egui::{Align, CornerRadius, Layout, Rect, Sense, Ui, WidgetInfo, vec2};
use toolbox_core::features::toggles::summary::summary;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{ButtonKind, Select, pill_button, rich};

use super::{SetupPage, Stage};
use crate::command::UiCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::toggles::Steps;

const NARROW_CONTROL: f32 = toolbox_gui_kit::tokens::Sizes::DEFAULT.setting_control_narrow;

/// Thin, so it reads as a mark of where the assistant stands, not as a
/// control.
const BAR_HEIGHT: f32 = 4.0;
const BAR_RADIUS: u8 = 2;

pub fn stages(ctx: &egui::Context, steps: &Steps) -> Vec<(Stage, String)> {
    let numbered = steps.pairs().enumerate().map(|(index, (id, title))| {
        let number = (index + 1).to_string();
        let name = i18n::text_with(
            ctx,
            "setup-step-item",
            &[("number", &number), ("title", title)],
        );

        (Stage::Step(id.to_owned()), name.into_owned())
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
    progress(ui, page, stages, current.unwrap_or_default());
    ui.add_space(ui.tokens().sizes.control_gap);
    ui.horizontal(|ui| {
        let back = ui
            .enabled_ui(current.unwrap_or_default() > 0, |ui| {
                pill_button(ui, &text("setup-back"), ButtonKind::Soft)
                    .on_hover_text(text("setup-back-key"))
            })
            .inner;

        if back.clicked() {
            moved |= go(page, stages, current, false);
        }

        // Next at the right edge, the list centred under the page, so it
        // stays put while Next changes its words.
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let next = next_label(context, page);
            let pressed = pill_button(ui, &text(next), ButtonKind::Primary)
                .on_hover_text(text("setup-next-key"));

            if pressed.clicked() {
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
                .value
                .and_then(|index| stages.get(index))
            {
                page.stage = stage.clone();
                moved = true;
            }
        });
    });

    moved
}

/// The stage's name or "Step N of M" over a thin bar, a progress indicator
/// for screen readers.
fn progress(ui: &mut Ui, page: &SetupPage, stages: &[(Stage, String)], at: usize) {
    let ctx = ui.ctx().clone();
    let tokens = ui.tokens();

    // The welcome page stands before the steps, the summary after.
    let steps = stages.len().saturating_sub(2);
    let words = match page.stage {
        Stage::Step(_) => {
            let (number, count) = (at.to_string(), steps.to_string());
            i18n::text_with(
                &ctx,
                "setup-progress",
                &[("number", &number), ("count", &count)],
            )
            .into_owned()
        }
        Stage::Welcome | Stage::Summary | Stage::Success => stages
            .get(at)
            .map(|(_, name)| name.clone())
            .unwrap_or_default(),
    };

    ui.label(rich(ui, &words, Weight::Regular, SMALL, tokens.secondary));
    ui.add_space(tokens.sizes.tight_gap);

    let size = vec2(ui.available_width(), BAR_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(Role::ProgressIndicator, true, &words));
    let (value, most) = filled(&page.stage, at, steps);
    let (value, most) = (whole(value), whole(most));
    ctx.accesskit_node_builder(response.id, |node| {
        node.set_numeric_value(value);
        node.set_min_numeric_value(0.0);
        node.set_max_numeric_value(most);
    });

    if !ui.is_rect_visible(rect) {
        return;
    }

    let share = if most > 0.0_f64 {
        value / most
    } else {
        0.0_f64
    };

    let filled = Rect::from_min_size(
        rect.min,
        vec2(rect.width() * share_of(share), rect.height()),
    );

    let radius = CornerRadius::same(BAR_RADIUS);
    ui.painter().rect_filled(rect, radius, tokens.field);
    ui.painter().rect_filled(filled, radius, tokens.accent);
}

/// How many of `steps` the bar fills at stage `at` of the list, so it
/// says what "Step N of M" says and is full from the last step on.
const fn filled(stage: &Stage, at: usize, steps: usize) -> (usize, usize) {
    match stage {
        Stage::Welcome => (0, steps),
        Stage::Step(_) => (if at < steps { at } else { steps }, steps),
        Stage::Summary | Stage::Success => (steps, steps),
    }
}

/// A stage count as accesskit takes it.
fn whole(count: usize) -> f64 {
    f64::from(u32::try_from(count).unwrap_or(u32::MAX))
}

/// `share` as the painter takes it, between none and all.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "a share between 0 and 1 loses nothing a pixel shows"
)]
const fn share_of(share: f64) -> f32 {
    (share as f32).clamp(0.0, 1.0)
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
        // No apply ran, so an earlier one has nothing to say here.
        page.ending = None;
        page.applied = None;
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
