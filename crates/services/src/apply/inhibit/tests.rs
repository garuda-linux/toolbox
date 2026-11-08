use toolbox_plan::protocol::Mode;

use super::{Hold, Inhibitor};

/// A dry run never reaches logind, so it holds nothing even with the
/// logind inhibitor.
#[tokio::test]
async fn a_dry_run_holds_nothing_back() {
    let Hold { _lock: lock } = Inhibitor::Logind.hold(Mode::DryRun).await;

    assert!(lock.is_none());
}

#[tokio::test]
async fn the_test_inhibitor_holds_nothing_back() {
    let Hold { _lock: lock } = Inhibitor::Off.hold(Mode::Run).await;

    assert!(lock.is_none());
}
