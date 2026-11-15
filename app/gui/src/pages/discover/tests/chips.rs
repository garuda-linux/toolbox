//! The category chips behave as a toggle page's section chips do.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::DiscoverPage;
use super::offered;

#[test]
fn a_second_click_on_the_picked_category_shows_every_category_again() {
    let mut harness = offered().harness(DiscoverPage::default());
    harness.state_mut().page.category = Some("internet".to_owned());
    harness.run();

    harness
        .get_all_by_role(Role::Button)
        .find(|node| {
            node.accesskit_node()
                .label()
                .is_some_and(|label| label.starts_with("Internet "))
        })
        .expect("the Internet chip")
        .click();
    harness.run();

    assert_eq!(harness.state().page.category, None);
}
