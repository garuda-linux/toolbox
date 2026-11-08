use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::{Page, page};

fn cut_at_the_top(harness: &Page) -> Vec<String> {
    let open = harness
        .get_by_role_and_label(Role::Button, "Open Kernels")
        .rect();

    harness
        .query_all_by_role(Role::Link)
        .filter(|link| link.rect().right() < open.left())
        .filter(|link| {
            link.rect().top() < open.top() - 0.5 && link.rect().bottom() > open.top() + 0.5
        })
        .filter_map(|link| link.accesskit_node().label())
        .collect()
}

/// Opening a page far down the list scrolls it into view. The list then
/// starts on a whole entry, never one cut through its text, whatever the
/// window's height.
#[test]
fn the_list_scrolled_to_the_open_page_starts_on_a_whole_entry() {
    for height in (600_u16..=900).step_by(9).map(f32::from) {
        let mut harness = page();
        harness.set_size(vec2(1024.0, height));
        harness.run();
        harness.state_mut().page.show("kernels.md", None);
        harness.run();

        let cut = cut_at_the_top(&harness);
        assert!(
            cut.is_empty(),
            "{height} px tall, cut at the list's top: {cut:?}"
        );

        let open = harness
            .query_all_by_role_and_label(Role::Link, "Kernels")
            .map(|link| link.rect())
            .next()
            .expect("Kernels is listed");

        assert!(
            open.bottom() <= height,
            "{height} px tall, the open page's entry ends at {}",
            open.bottom()
        );
    }
}
