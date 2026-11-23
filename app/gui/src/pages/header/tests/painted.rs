//! The toolbar's groups stand 8 px apart as the eye sees them, measured
//! between what each control paints, not between their widget rects. A
//! field and a dropdown paint inset from their rect, Read again only its
//! glyph. Tabs count as their pills, inset from their rects, as an unchosen
//! tab paints only its text until the pointer or the keyboard is on it.

use std::sync::Arc;

use egui::accesskit::Role;
use egui::{Rect, Shape, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::Snapshot;
use toolbox_gui_kit::widgets::{icon_button_padding, tab_pill_inset};

use super::{GROUP_GAP, MEDIUM, SMALLEST, WIDE, same_row, services_header, sort_caption};
use crate::pages::locales::LocalesPage;
use crate::testing::{PageFixture, PageState, rect_of};

/// What a test sees of one group: its name and the part it paints.
type Group = (&'static str, Rect);

/// Every shape the last frame painted, those inside a `Shape::Vec` included.
/// gui-kit's own walk is crate private.
fn shapes<'frame, S>(harness: &'frame Harness<'_, S>) -> Vec<&'frame Shape> {
    fn walk<'shape>(shape: &'shape Shape, found: &mut Vec<&'shape Shape>) {
        if let Shape::Vec(inner) = shape {
            for each in inner {
                walk(each, found);
            }
        } else {
            found.push(shape);
        }
    }

    let mut found = Vec::new();
    harness
        .output()
        .shapes
        .iter()
        .for_each(|clipped| walk(&clipped.shape, &mut found));

    found
}

/// What the eye takes for a shape: a shadow is a soft halo under the
/// control, a clear rect without a stroke shows nothing.
fn seen(shape: &Shape) -> Option<Rect> {
    if let Shape::Rect(rect) = shape {
        let unseen = rect.fill.a() == 0 && rect.stroke.is_empty();
        if rect.blur_width > 0.0 || unseen {
            return None;
        }
    }

    let bounds = shape.visual_bounding_rect();
    bounds.is_positive().then_some(bounds)
}

/// The union of the seen shapes that lie inside `rect`, the control's
/// widget rect. Shapes reaching past it belong to the band or the page.
fn painted_in<S>(harness: &Harness<'_, S>, rect: Rect) -> Rect {
    let reach = rect.expand(0.5);
    shapes(harness)
        .into_iter()
        .filter_map(seen)
        .filter(|bounds| reach.contains_rect(*bounds))
        .fold(Rect::NOTHING, Rect::union)
}

fn painted_node<S>(harness: &Harness<'_, S>, role: Role, label: &str) -> Rect {
    painted_in(harness, rect_of(harness, role, label))
}

/// The header's Read again, named Reading while a read runs. A failure
/// below the header shows a second one lower down.
fn read_again<S>(harness: &Harness<'_, S>) -> Rect {
    let header = harness
        .query_all_by_role(Role::Button)
        .filter(|node| {
            let label = node.accesskit_node().label();
            matches!(label.as_deref(), Some("Read Again" | "Reading"))
        })
        .map(|node| node.rect())
        .min_by(|one, other| one.top().total_cmp(&other.top()))
        .expect("the header shows Read again");

    painted_in(harness, header)
}

/// The tabs from the first pill's left to the last pill's right.
fn tabs<S>(harness: &Harness<'_, S>, labels: [&str; 2]) -> Rect {
    labels
        .iter()
        .map(|label| rect_of(harness, Role::RadioButton, label))
        .fold(Rect::NOTHING, Rect::union)
        .shrink2(vec2(tab_pill_inset(), 0.0))
}

/// The sort group runs from its caption to the end of its dropdown.
fn sort<S>(harness: &Harness<'_, S>, label: &str) -> Rect {
    sort_caption(harness).union(painted_node(harness, Role::ComboBox, label))
}

/// The painted gaps between neighbouring groups of one row, row by row and
/// left to right, each one that is not a group gap told as a miss.
fn misses(page: &str, width: f32, mut groups: Vec<Group>) -> Vec<String> {
    groups.sort_by(|(_, one), (_, other)| {
        if same_row(*one, *other) {
            one.left().total_cmp(&other.left())
        } else {
            one.center().y.total_cmp(&other.center().y)
        }
    });

    groups
        .windows(2)
        .filter_map(|pair| match pair {
            [(left_name, left), (right_name, right)] if same_row(*left, *right) => {
                Some((*left_name, *right_name, right.left() - left.right()))
            }
            _ => None,
        })
        .filter(|(left, right, gap)| (gap - wanted(left, right)).abs() > 0.5)
        .map(|(left, right, gap)| {
            let wanted = wanted(left, right);
            format!("{page} at {width}: {left} to {right} paints {gap} px, wanted {wanted}")
        })
        .collect()
}

/// The painted gap wanted between two neighbouring groups.
fn wanted(left: &str, right: &str) -> f32 {
    match (left, right) {
        // 8 px painted would overlap the rects, so they touch instead.
        ("tabs", "refresh") => tab_pill_inset() + icon_button_padding(),
        _ => GROUP_GAP,
    }
}

#[track_caller]
fn assert_none(misses: &[String]) {
    assert!(
        misses.is_empty(),
        "wrong painted gaps between groups:\n{}",
        misses.join("\n")
    );
}

fn locales(width: f32, packs: bool) -> Harness<'static, PageState<LocalesPage>> {
    let mut harness = PageFixture {
        snapshot: Arc::new(Snapshot::default()),
        ..PageFixture::default()
    }
    .harness(LocalesPage::default());

    harness.set_size(vec2(width, 900.0));
    harness.run();
    if packs {
        harness
            .get_by_role_and_label(Role::RadioButton, "Language Packs")
            .click();
        harness.run();
    }

    harness
}

#[test]
fn the_services_toolbar_paints_its_groups_eight_px_apart() {
    let mut missed = Vec::new();
    for width in [WIDE, MEDIUM, SMALLEST] {
        let harness = services_header(width);
        let groups = vec![
            (
                "search",
                painted_node(&harness, Role::TextInput, "Search services"),
            ),
            ("sort", sort(&harness, "Sort services")),
            ("tabs", tabs(&harness, ["System", "User"])),
            ("refresh", read_again(&harness)),
        ];

        missed.extend(misses("Services", width, groups));
    }

    assert_none(&missed);
}

#[test]
fn the_locales_toolbar_paints_its_groups_eight_px_apart_on_either_tab() {
    let mut missed = Vec::new();
    for width in [WIDE, MEDIUM, SMALLEST] {
        for packs in [false, true] {
            let harness = locales(width, packs);
            let search = painted_node(&harness, Role::TextInput, "Search locales and packs");
            let mut groups = vec![
                ("search", search),
                ("tabs", tabs(&harness, ["Locales", "Language Packs"])),
                ("refresh", read_again(&harness)),
            ];

            if packs {
                groups.push(("sort", sort(&harness, "Sort language packs")));
            }

            missed.extend(misses("Locales", width, groups));
        }
    }

    assert_none(&missed);
}
