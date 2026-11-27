use egui_kittest::Harness;
use toolbox_gui_kit::testing::{filled_rects, lines_in, painted_text_within, rims_in};

use super::{Page, page_on};

/// The command palette's page, tall enough that every table shows.
fn tables() -> Page {
    let mut harness = page_on("command-palette.md");
    harness.set_size(egui::vec2(1024.0, 3000.0));
    harness.run();
    harness
}

fn outline<State>(harness: &Harness<'_, State>) -> egui::Color32 {
    toolbox_gui_kit::tokens::current(&harness.ctx).outline
}

#[test]
fn stripes_reach_the_frame_and_a_rule_closes_the_header() {
    let harness = tables();
    let stripe = harness.ctx.global_style().visuals.faint_bg_color;
    let frames = rims_in(&harness, outline(&harness));
    let stripes: Vec<_> = filled_rects(&harness, stripe)
        .into_iter()
        .filter_map(|band| {
            frames
                .iter()
                .find(|(frame, _)| frame.expand(0.5).contains_rect(band))
                .map(|(frame, _)| (band, *frame))
        })
        .collect();

    assert!(!stripes.is_empty(), "no stripe in a table");
    for (band, frame) in &stripes {
        let flush =
            (frame.left() - band.left()).abs() < 1.5 && (frame.right() - band.right()).abs() < 1.5;

        assert!(flush, "{band:?} stops short of its frame {frame:?}");
    }

    let rules = lines_in(&harness, outline(&harness));
    assert!(
        stripes
            .iter()
            .all(|(_, frame)| rules.iter().any(|[from, to]| {
                (from.x - frame.left()).abs() < 1.5 && (to.x - frame.right()).abs() < 1.5
            })),
        "a table without a rule under its header row"
    );
}

#[test]
fn code_in_a_table_has_no_fill_of_its_own() {
    let harness = tables();

    // The paragraph above the table names Esc too, with its chip.
    let code = rims_in(&harness, outline(&harness))
        .into_iter()
        .find_map(|(frame, _)| painted_text_within(&harness, "Esc", frame))
        .expect("the table names Esc");

    assert!(
        code.galley
            .job
            .sections
            .iter()
            .all(|section| section.format.background == egui::Color32::TRANSPARENT)
    );
}

/// Every cell starts at its row's top, and a row's stripe holds its text.
/// The Home page's table wraps its long cells at this width.
#[test]
fn cells_start_at_their_rows_top_inside_their_stripe() {
    let mut harness = page_on("home.md");
    harness.set_size(egui::vec2(980.0, 3000.0));
    harness.run();

    let text = |words: &str| {
        toolbox_gui_kit::testing::painted_text(&harness, words).map_or_else(
            || panic!("{words} is painted"),
            |shape| shape.galley.rect.translate(shape.pos.to_vec2()),
        )
    };

    let (tile, needs) = (text("Updates"), text("Any update is available"));
    assert!(
        (tile.top() - needs.top()).abs() < 1.0,
        "Updates at {}, its neighbour at {}",
        tile.top(),
        needs.top()
    );

    let (next, beside) = (text("Disk"), text("The space is low, see below"));
    assert!(
        (next.top() - beside.top()).abs() < 1.0,
        "{next:?} {beside:?}"
    );

    let stripe = harness.ctx.global_style().visuals.faint_bg_color;
    let band = filled_rects(&harness, stripe)
        .into_iter()
        .find(|band| band.contains(tile.center()))
        .expect("a stripe under the first body row");

    assert!(
        band.contains_rect(tile) && band.contains_rect(needs),
        "{band:?} {tile:?} {needs:?}"
    );
    assert!(
        band.bottom() <= next.top(),
        "{band:?} runs into the next row at {}",
        next.top()
    );
}
