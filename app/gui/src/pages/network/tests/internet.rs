use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use toolbox_gui_kit::testing::circle_fills;
use toolbox_gui_kit::tokens;

use super::page;
use crate::pages::Refusals;

#[test]
fn the_internet_state_is_a_dot_and_its_word_where_the_values_are() {
    let harness = page(Refusals::default());
    let title = harness
        .get_by_role_and_label(Role::Label, "Internet")
        .rect();

    let state = harness.get_by_label("Connected").rect();
    let router = harness.get_by_label("192.168.1.1").rect();

    assert!(
        (state.right() - router.right()).abs() < 0.5,
        "Connected ends at {}, the router's address at {}",
        state.right(),
        router.right()
    );
    assert!(
        (state.center().y - title.center().y).abs() < 2.0,
        "Connected at {state:?} is not on the title's line at {title:?}"
    );

    let success = tokens::current(&harness.ctx).success;
    assert!(
        circle_fills(&harness).contains(&success),
        "no green dot before Connected"
    );
}
