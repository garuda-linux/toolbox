use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Toggle;
use toolbox_core::domains::Packages;
use toolbox_gui_kit::testing::filled_rect;
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{dim_opacity, hue_of};
use toolbox_plan::PackageName;

use super::super::TogglePage;
use super::{Plain, shipped};
use crate::catalogs::CatalogId;
use crate::pages::rows::PICTURE_SIZE;
use crate::testing::{PageFixture, PageState, assert_text_beside_picture};

/// Off by less than this, two edges count as one.
const SAME: f32 = 0.5;

type Page = Harness<'static, PageState<Plain>>;

/// One row to measure: what the search finds it by, its title, its
/// description, and whether its tile is faded.
struct Row {
    search: &'static str,
    title: &'static str,
    description: &'static str,
    faded: bool,
}

fn page(fixture: PageFixture, search: &str) -> Page {
    let mut page = TogglePage::new(CatalogId::Components, "page-gaming", "page-gaming-subtitle");
    search.clone_into(&mut page.search);
    let mut harness = fixture.harness(Plain(page));
    harness.run();
    harness
}

fn tile_of(harness: &Page, row: &Row) -> Rect {
    let tokens = tokens::current(&harness.ctx);
    let fill = tokens.tiles.letters().of(hue_of(row.search)).fill;
    let fill = if row.faded {
        fill.gamma_multiply(dim_opacity(&tokens))
    } else {
        fill
    };

    let tile = filled_rect(harness, fill).expect("the letter tile is painted");
    assert!((tile.width() - PICTURE_SIZE).abs() < SAME, "tile {tile:?}");
    tile
}

fn assert_lefts(harness: &Page, row: &Row, state: &str) {
    let picture = tile_of(harness, row);
    assert_text_beside_picture(harness, picture, (row.title, row.description), state);
}

fn assert_in_every_state(mut harness: Page, row: &Row) {
    assert_lefts(&harness, row, "at rest");

    let node = row_named(&harness, row.title);
    let middle = node.rect().center();
    harness.hover_at(middle);
    harness.run();
    assert_lefts(&harness, row, "hovered");

    row_named(&harness, row.title).focus();
    harness.run();
    assert_lefts(&harness, row, "focused");
}

#[test]
fn a_title_with_a_chip_starts_where_its_description_does_right_of_the_tile() {
    // JamesDSP needs PipeWire, which the empty system lacks, so its title
    // carries a chip and its tile fades.
    let row = Row {
        search: "jamesdsp",
        title: "JamesDSP",
        description: "An audio effect processor for adding bass boost, equalizers, and other \
                      effects. It is recommended to try EasyEffects first.",
        faded: true,
    };

    assert_in_every_state(page(shipped(), row.search), &row);
}

#[test]
fn a_queued_title_starts_where_its_description_does() {
    let mut fixture = shipped();
    fixture.changes.want::<Packages>(
        PackageName::new("calf").expect("a package name"),
        Toggle::On,
        &toolbox_core::Snapshot::default(),
    );

    let row = Row {
        search: "calf",
        title: "Calf plugins",
        description: "Calf Studio Gear. A high-quality set of audio processing plugins. Install \
                      this to give EasyEffects more professional studio effects.",
        faded: false,
    };

    let harness = page(fixture, row.search);
    assert!(harness.query_by_label("Turns on with Apply").is_some());
    assert_in_every_state(harness, &row);
}

/// The row whose spoken name starts with `title`, which goes on with the
/// note beside it.
fn row_named<'tree>(harness: &'tree Page, title: &str) -> egui_kittest::Node<'tree> {
    harness
        .query_all_by_role(Role::ListItem)
        .find(|node| {
            node.accesskit_node()
                .label()
                .is_some_and(|label| label.starts_with(title))
        })
        .unwrap_or_else(|| panic!("no row named {title}"))
}
