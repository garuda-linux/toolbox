use egui::Rect;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_catalog::ToggleCatalog;
use toolbox_gui_kit::testing::{filled_rect, painted_text};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::hue_of;

use super::super::TogglePage;
use super::Plain;
use crate::catalogs::{CatalogId, Catalogs, Toggles};
use crate::testing::{PageFixture, PageState, letters_top};

/// Off by less than this, two edges count as one.
const SAME: f32 = 0.5;

/// How far the top of a title's letters may be from its capitals'.
pub(super) const LETTERS: f32 = 1.5;

/// Entries named after how many lines their description takes.
const CATALOG: &str = concat!(
    "[[section]]\nid = \"audio\"\ntitle = \"Audio\"\nicon = \"x\"\n",
    "[[section.entry]]\nid = \"one\"\ntitle = \"One line\"\n",
    "description = \"First\"\ncheck = { service = \"one.service\" }\n",
    "[[section.entry]]\nid = \"two\"\ntitle = \"Two lines\"\n",
    "description = \"First\\nSecond\"\ncheck = { service = \"two.service\" }\n",
    "[[section.entry]]\nid = \"three\"\ntitle = \"Three lines\"\n",
    "description = \"First\\nSecond\\nThird\"\ncheck = { service = \"three.service\" }\n",
);

struct Placed {
    tile: Rect,
    letters: f32,
    switch: Rect,
    content: Rect,
}

fn placed(id: &str, name: &str) -> Placed {
    let catalog = ToggleCatalog::parse(CATALOG, "test.toml").expect("a valid catalog");
    let fixture = PageFixture {
        catalogs: Catalogs {
            components: Toggles {
                catalog: Ok(catalog),
                ..Toggles::default()
            },
            ..Catalogs::default()
        },
        ..PageFixture::default()
    };

    let mut page = TogglePage::new(CatalogId::Components, "page-gaming", "page-gaming-subtitle");
    id.clone_into(&mut page.search);
    let mut harness: Harness<'static, PageState<Plain>> = fixture.harness(Plain(page));
    harness.run();

    let fill = tokens::current(&harness.ctx)
        .tiles
        .letters()
        .of(hue_of(id))
        .fill;

    let tile = filled_rect(&harness, fill).expect("the letter tile is painted");
    let shape = painted_text(&harness, name).expect("the title is painted");
    let text = shape.galley.rect.translate(shape.pos.to_vec2());
    let letters = letters_top(&mut harness, text).expect("the title shows light letters");
    let switch = harness.get_by_role_and_label(Role::CheckBox, name).rect();
    let row = harness.get_by_role(Role::ListItem).rect();
    let padding = tokens::current(&harness.ctx).sizes.row_padding.y;

    Placed {
        tile,
        letters,
        switch,
        content: row.shrink2(egui::vec2(0.0, padding)),
    }
}

#[test]
fn the_tile_starts_at_the_titles_letters_however_many_lines_follow() {
    for (id, title) in [
        ("one", "One line"),
        ("two", "Two lines"),
        ("three", "Three lines"),
    ] {
        let row = placed(id, title);

        // The "l" reaches a little above the capitals.
        assert!(
            (row.tile.top() - row.letters).abs() < LETTERS,
            "{id}: the tile starts at {}, the title's letters at {}",
            row.tile.top(),
            row.letters
        );
        assert!(
            row.letters >= row.content.top() - SAME,
            "{id}: the title's letters start at {}, above the row's content at {}",
            row.letters,
            row.content.top()
        );
        assert!(
            row.tile.bottom() <= row.content.bottom() + SAME,
            "{id}: the tile ends at {}, below the row's content at {}",
            row.tile.bottom(),
            row.content.bottom()
        );
        assert!(
            (row.switch.center().y - row.content.center().y).abs() < SAME,
            "{id}: the switch's middle is at {}, the row's at {}",
            row.switch.center().y,
            row.content.center().y
        );
    }
}
