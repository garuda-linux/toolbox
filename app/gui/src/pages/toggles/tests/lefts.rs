use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Toggle;
use toolbox_core::domains::Packages;
use toolbox_core::testing::Queue as _;
use toolbox_gui_kit::testing::textured_rects;
use toolbox_plan::PackageName;

use super::super::TogglePage;
use super::{Plain, shipped};
use crate::catalogs::CatalogId;
use crate::pages::rows::PICTURE_SIZE;
use crate::testing::{PageFixture, PageState, assert_text_beside_picture};

/// Off by less than this, two edges count as one.
const SAME: f32 = 0.5;
/// The glyph's edge, Kirigami's smallMedium icon.
const ICON: f32 = 22.0;

type Page = Harness<'static, PageState<Plain>>;

/// One row to measure: what the search finds it by, its title, its
/// description.
struct Row {
    search: &'static str,
    title: &'static str,
    description: &'static str,
}

fn page(fixture: PageFixture, search: &str) -> Page {
    let mut page = TogglePage::new(CatalogId::Components, "page-gaming", "page-gaming-subtitle");
    search.clone_into(&mut page.search);
    let mut harness = fixture.harness(Plain(page));
    harness.run();
    harness
}

/// The picture slot of a row: the square around its centred glyph.
fn tile_of(harness: &Page) -> Rect {
    let glyph = textured_rects(harness)
        .into_iter()
        .find(|rect| (rect.width() - ICON).abs() < SAME)
        .expect("the glyph is painted");

    Rect::from_center_size(glyph.center(), egui::vec2(PICTURE_SIZE, PICTURE_SIZE))
}

fn assert_lefts(harness: &Page, row: &Row, state: &str) {
    let picture = tile_of(harness);
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
    // carries a chip.
    let row = Row {
        search: "jamesdsp",
        title: "JamesDSP",
        description: "An audio effect processor for adding bass boost, equalizers, and other \
                      effects. It is recommended to try EasyEffects first.",
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
    };

    let harness = page(fixture, row.search);
    assert!(harness.query_by_label("Installs with Apply").is_some());
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
