//! The details beside the list: a row opens them, Escape and Back close
//! them, a narrow page shows them alone, and the text holds at the narrowest
//! two-column width in both languages.

use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Key, Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::features::packages::list::Catalog;
use toolbox_core::features::packages::{IndexEntry, Origin, PackageIndex};
use toolbox_gui_kit::testing::{lines_in, painted_text_within};
use toolbox_gui_kit::tokens;
use toolbox_plan::PackageName;

use super::super::DiscoverPage;
use super::{DESCRIPTION, focused_firefox, offered, searching_firefox};
use crate::packages::{Packages, Read};
use crate::testing::{PageState, assert_rows_hold_their_parts};

const BACK: &str = "Back to App Catalog";

pub(super) type Harnessed = Harness<'static, PageState<DiscoverPage>>;

/// Clicks the widget named `label` and runs the frame the click changes.
fn click(harness: &mut Harnessed, role: Role, label: &str) {
    harness.get_by_role_and_label(role, label).click();
    harness.run();
}

/// Firefox and Floorp listed, Firefox's details open by a click on its row.
pub(super) fn open_firefox() -> Harnessed {
    let mut harness = offered().harness(searching_firefox());
    harness.run();
    click(&mut harness, Role::ListItem, "Firefox");
    harness
}

pub(super) fn open_id(harness: &Harnessed) -> Option<&str> {
    harness
        .state()
        .page
        .open
        .as_ref()
        .map(|open| open.id.as_str())
}

fn lists_floorp(harness: &Harnessed) -> bool {
    harness
        .query_by_role_and_label(Role::ListItem, "Floorp")
        .is_some()
}

#[test]
fn a_click_on_a_row_opens_its_details_beside_the_list() {
    let harness = open_firefox();

    assert_eq!(open_id(&harness), Some("firefox"));
    assert!(lists_floorp(&harness), "the list stays beside the details");
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Firefox")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Internet")
            .is_some(),
        "the category headings stay above the list"
    );

    // The details hold the one install button, the rows beside them none.
    assert_eq!(
        harness
            .query_all_by_role_and_label(Role::Button, "Install Firefox")
            .count(),
        1
    );
}

#[test]
fn escape_closes_the_details_and_the_list_takes_its_full_width() {
    let mut harness = open_firefox();

    harness.key_press(Key::Escape);
    harness.run();
    harness.run();

    assert!(harness.state().page.open.is_none());
    assert!(
        harness
            .query_by_role_and_label(Role::Button, BACK)
            .is_none()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Install Firefox")
            .is_some(),
        "the rows show their buttons again"
    );
}

#[test]
fn a_wide_page_has_no_back_button_beside_the_list() {
    let harness = open_firefox();

    assert!(lists_floorp(&harness));
    assert!(
        harness
            .query_by_role_and_label(Role::Button, BACK)
            .is_none(),
        "the list stays beside the details, so Escape is the way back"
    );
}

#[test]
fn a_narrow_page_shows_the_details_alone_and_back_returns_to_the_list() {
    let mut harness = open_firefox();

    harness.set_size(vec2(480.0, 480.0));
    harness.run();
    assert!(!lists_floorp(&harness), "the details replace the list");
    assert!(
        harness
            .query_by_role_and_label(Role::Button, BACK)
            .is_some()
    );

    click(&mut harness, Role::Button, BACK);

    assert!(harness.state().page.open.is_none());
    assert!(lists_floorp(&harness));
}

#[test]
fn enter_on_a_focused_row_opens_its_details() {
    let mut harness = focused_firefox();

    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(open_id(&harness), Some("firefox"));
}

#[test]
fn the_arrow_keys_move_the_focus_and_the_details_follow_it() {
    let mut harness = open_firefox();

    harness.key_press(Key::ArrowDown);
    harness.run();

    assert_eq!(open_id(&harness), Some("floorp"));
}

/// The details name each package with its repository when the package
/// index knows it.
#[test]
fn the_details_name_each_package_with_its_repository() {
    let mut fixture = offered();
    fixture.services.packages = Packages::showing(Read::Known(firefox_in_extra()), None);

    let mut harness = fixture.harness(searching_firefox());
    harness.run();
    click(&mut harness, Role::ListItem, "Firefox");

    assert!(
        painted_text_within(&harness, "firefox from extra", harness.ctx.content_rect()).is_some()
    );
}

fn firefox_in_extra() -> Arc<Catalog> {
    Arc::new(Catalog::new(PackageIndex {
        entries: vec![IndexEntry {
            name: PackageName::new("firefox").unwrap(),
            description: toolbox_text::plain("Browser"),
            version: "1.0-1".to_owned(),
            origin: Origin::Repository("extra".to_owned()),
            size: 1024,
            installed: None,
        }],
        local: Vec::new(),
    }))
}

/// The narrowest window that still shows the list beside the details, found
/// by halving the widths between a narrow and a wide window.
fn narrowest_two_columns(harness: &mut Harnessed) -> f32 {
    let (mut narrow, mut wide) = (300.0_f32, 1200.0_f32);
    for _ in 0..12_u32 {
        let middle = f32::midpoint(narrow, wide);
        harness.set_size(vec2(middle, 800.0));
        harness.run();
        if lists_floorp(harness) {
            wide = middle;
        } else {
            narrow = middle;
        }
    }

    harness.set_size(vec2(wide, 800.0));
    harness.run();
    wide
}

/// Asserts that `text` is painted whole inside `within`. A row's one-line
/// summary is cut on purpose, so the details' texts are checked alone.
fn assert_whole(harness: &Harnessed, text: &str, within: Rect, language: &str) {
    let shape = painted_text_within(harness, text, within)
        .unwrap_or_else(|| panic!("{language}: {text} is not painted in {within:?}"));

    assert!(!shape.galley.elided, "{language}: {text} is cut");
    let rect = shape.galley.rect.translate(shape.pos.to_vec2());
    assert!(
        rect.right() <= within.right(),
        "{language}: {text} at {rect:?} leaves {within:?}"
    );
}

/// The window's halves at the vertical line between the columns: the list
/// left of it, the details from it to the right edge.
fn columns(harness: &Harnessed) -> (Rect, Rect) {
    let window = harness.ctx.content_rect();
    let separator = tokens::current(&harness.ctx).separator;
    let x = lines_in(harness, separator)
        .iter()
        .find(|[from, to]| (from.x - to.x).abs() < 0.01)
        .map(|[from, _]| from.x)
        .expect("the line between the columns");

    let list = Rect::from_min_max(window.min, pos2(x, window.bottom()));
    let details = Rect::from_min_max(pos2(x, window.top()), window.max);
    (list, details)
}

#[test]
fn the_list_and_the_details_hold_their_text_at_the_narrowest_two_column_width() {
    let cases = [
        (
            "en-US",
            [
                "Firefox",
                DESCRIPTION,
                "Install",
                "Packages",
                "firefox",
                "Website",
                "Open Website",
            ],
        ),
        (
            "de-DE",
            [
                "Firefox",
                DESCRIPTION,
                "Installieren",
                "Pakete",
                "firefox",
                "Webseite",
                "Website öffnen",
            ],
        ),
    ];

    for (language, texts) in cases {
        let mut harness = offered().harness(searching_firefox());
        let tag = language.parse().expect("a language tag");
        crate::i18n::set_language(&harness.ctx, &tag);
        harness.run();
        click(&mut harness, Role::ListItem, "Firefox");

        let width = narrowest_two_columns(&mut harness);
        assert!(width < 1200.0, "{language}: no two-column width found");

        assert_rows_hold_their_parts(&harness);
        let (list, details) = columns(&harness);
        assert_whole(&harness, "Firefox", list, language);
        for text in texts {
            assert_whole(&harness, text, details, language);
        }
    }
}
