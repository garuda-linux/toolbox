//! An Apply whose fresh read of the build orders breaks.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::runtime::Runtime;
use toolbox_plan::PackageName;
use toolbox_services::testing::until;
use toolbox_state::aur::order::AurOrder;
use toolbox_state::testing::aur::order::OCTOPI_GIT_STDOUT;
use toolbox_system::testing::aur_orders::FakeAurOrders;
use toolbox_system::{AurOrderSource, ProbeError};

use super::orders::{app_with, click_apply, plan_sent, queued, run_of};

/// Answers the first read, then breaks as a job that panics does.
struct BreaksLater {
    orders: FakeAurOrders,
    asked: AtomicUsize,
}

impl AurOrderSource for BreaksLater {
    fn order(&self, package: &PackageName) -> Result<AurOrder, ProbeError> {
        assert!(
            self.asked.fetch_add(1, Ordering::SeqCst) == 0,
            "the second read breaks"
        );

        self.orders.order(package)
    }
}

/// An order of an earlier read never builds: the plan would name a run the
/// user did not just ask for.
#[test]
fn an_apply_whose_fresh_read_breaks_sends_no_plan_and_drops_the_old_order() {
    let runtime = Runtime::new().unwrap();
    let source: Arc<dyn AurOrderSource> = Arc::new(BreaksLater {
        orders: FakeAurOrders::new(vec![("octopi-git", Ok(OCTOPI_GIT_STDOUT))]),
        asked: AtomicUsize::new(0),
    });

    let mut harness = app_with(&runtime, source, queued(&["octopi-git"]));
    until("the first read never landed", || {
        harness.step();
        run_of(&harness, "octopi-git").is_some()
    });

    click_apply(&mut harness);
    until("the broken read never landed", || {
        harness.step();
        run_of(&harness, "octopi-git").is_none()
    });

    for _ in 0_u8..5 {
        harness.step();
    }

    assert!(!plan_sent(&harness));
}
