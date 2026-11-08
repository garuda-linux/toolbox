mod artwork;
mod lefts;
mod pictures;
mod tops;

use std::path::Path;

use egui::accesskit::Role;
use egui::{Key, Modifiers, Ui};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Toggle;

use super::{NoLead, TogglePage};
use crate::catalogs::{CatalogId, Catalogs};
use crate::command::{AppCommand, Command};
use crate::pages::{PageContext, PageView};
use crate::testing::{PageFixture, PageState, assert_header, rect_of};

/// A toggles page without lead sections, as the shell would draw one.
struct Plain(TogglePage);

impl PageView for Plain {
    fn header(&mut self, ui: &mut Ui, _context: &PageContext<'_>) -> bool {
        self.0.header(ui);
        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        self.0.ui(ui, context, &NoLead);
    }
}

fn shipped() -> PageFixture {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    PageFixture {
        catalogs: Catalogs::load(&dir),
        ..PageFixture::default()
    }
}

fn page(catalog: CatalogId, search: &str) -> Harness<'static, PageState<Plain>> {
    let mut page = TogglePage::new(catalog, "page-gaming", "page-gaming-subtitle");
    search.clone_into(&mut page.search);
    let mut harness = shipped().harness(Plain(page));
    harness.run();
    harness
}

fn right_click(harness: &mut Harness<'static, PageState<Plain>>, row: &str) {
    let at = harness
        .get_by_role_and_label(Role::ListItem, row)
        .rect()
        .left_top()
        + egui::vec2(8.0, 8.0);

    harness.hover_at(at);
    for pressed in [true, false] {
        harness.event(egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Secondary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }

    harness.run();
}

#[test]
fn space_on_a_focused_entry_flips_its_switch() {
    let mut harness = page(CatalogId::Components, "");
    harness
        .get_by_role_and_label(Role::ListItem, "PipeWire")
        .focus();
    harness.run();

    harness.key_press(Key::Space);
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(AppCommand::ToggleEntry {
            catalog: CatalogId::Components,
            entry: "pipewire-support".to_owned(),
            wish: Toggle::On,
        })]
    );
}

#[test]
fn a_right_click_offers_the_website_and_the_name() {
    let mut harness = page(CatalogId::Gaming, "0 a.d.");

    right_click(&mut harness, "0 A.D.");

    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Open website")
            .is_some()
    );
    harness
        .get_by_role_and_label(Role::MenuItem, "Copy name")
        .click();
    harness.run();
    assert_eq!(harness.state().fixture.sent().len(), 1, "the note");
}

#[test]
fn an_entry_without_a_website_offers_only_its_name() {
    let mut harness = page(CatalogId::Components, "");

    right_click(&mut harness, "PipeWire");

    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Copy name")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::MenuItem, "Open website")
            .is_none()
    );
}

#[test]
fn a_frame_with_the_same_inputs_builds_no_rows() {
    let mut harness = page(CatalogId::Components, "");
    let built = harness.state().page.0.listed.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.0.listed.builds(), built);

    "pipe".clone_into(&mut harness.state_mut().page.0.search);
    harness.run();
    assert_eq!(harness.state().page.0.listed.builds(), built + 1);
}

#[test]
fn the_search_starts_the_toolbar_under_the_header() {
    let harness = page(CatalogId::Gaming, "");
    let search = rect_of(&harness, Role::TextInput, "Search this page");
    let texts = (
        "Gaming",
        "Launchers, tools and emulators for playing on Linux. Pick what you want, then apply.",
    );

    assert_header(&harness, texts, None, Some(search));
}

/// Each chip's name and its "on/total" count, the All chip first.
fn chip_counts(harness: &Harness<'static, PageState<Plain>>) -> Vec<(String, usize, usize)> {
    harness
        .query_all_by_role(Role::Button)
        .filter_map(|button| {
            let label = button.accesskit_node().label()?;
            let (name, count) = label.rsplit_once(' ')?;
            let (on, total) = count.split_once('/')?;
            Some((name.to_owned(), on.parse().ok()?, total.parse().ok()?))
        })
        .collect()
}

#[test]
fn the_all_chip_counts_every_section_and_follows_the_search() {
    for search in ["", "pipe"] {
        let harness = page(CatalogId::Components, search);
        let chips = chip_counts(&harness);
        let (all, sections) = chips.split_first().expect("chips");

        assert_eq!(all.0, "All", "{search:?}: {chips:?}");
        let on = sections.iter().map(|chip| chip.1).sum::<usize>();
        let total = sections.iter().map(|chip| chip.2).sum::<usize>();
        assert_eq!((all.1, all.2), (on, total), "{search:?}: {chips:?}");
    }
}

#[test]
fn a_section_heading_counts_as_its_chip_does() {
    let harness = page(CatalogId::Components, "");

    assert_eq!(heading_count(&harness, "Audio").as_deref(), Some("0/8"));
}

#[test]
fn during_a_search_a_section_heading_counts_its_matches_as_its_chip_does() {
    let harness = page(CatalogId::Components, "pipe");
    let chips = chip_counts(&harness);
    let (_, on, total) = chips
        .iter()
        .find(|chip| chip.0 == "Audio")
        .expect("the Audio chip");

    assert!(*total < 8, "the search keeps every entry: {chips:?}");
    assert_eq!(
        heading_count(&harness, "Audio"),
        Some(format!("{on}/{total}"))
    );
}

/// The count painted beside the heading `title`, below its chip.
fn heading_count(harness: &Harness<'static, PageState<Plain>>, title: &str) -> Option<String> {
    let painted: Vec<(String, egui::Rect)> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| {
            let egui::Shape::Text(text) = &clipped.shape else {
                return None;
            };

            let rect = text.galley.rect.translate(text.pos.to_vec2());
            Some((text.galley.text().to_owned(), rect))
        })
        .collect();

    // The chip reads the title too, the heading is the one below it.
    let heading = painted
        .iter()
        .filter(|(text, _)| text == title)
        .map(|(_, rect)| *rect)
        .max_by(|one, other| one.top().total_cmp(&other.top()))?;

    painted
        .iter()
        .find(|(_, rect)| {
            rect.left() > heading.right() && (rect.center().y - heading.center().y).abs() < 6.0
        })
        .map(|(text, _)| text.clone())
}
