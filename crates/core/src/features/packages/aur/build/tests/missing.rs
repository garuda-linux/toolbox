use toolbox_state::testing::aur::order::MISSING_STDOUT;
use toolbox_text::plain;

use super::*;

#[test]
fn a_missing_line_gives_no_plan_and_names_the_package() {
    let order = capture(MISSING_STDOUT);

    assert_eq!(
        plan(
            &queued(&["no-such-package-toolbox"]),
            &[order],
            &run(),
            &none()
        ),
        Err(Missing {
            paru: vec![plain("no-such-package-toolbox")],
            unordered: queued(&["no-such-package-toolbox"]),
        })
    );
}

#[test]
fn a_queued_name_in_no_order_gives_no_plan_and_names_the_package() {
    assert_eq!(
        plan(
            &queued(&["octopi-git", "not-in-any-order"]),
            std::slice::from_ref(&octopi()),
            &run(),
            &none()
        ),
        Err(Missing {
            paru: Vec::new(),
            unordered: queued(&["not-in-any-order"]),
        })
    );
}

#[test]
fn an_empty_queue_plans_nothing() {
    assert_eq!(plan(&[], &[], &run(), &none()), Ok(BuildPlan::default()));
}
