use super::super::PackagesPage;
use super::fixture;

#[test]
fn a_frame_with_the_same_status_builds_no_package_view() {
    let mut harness = fixture(None).harness(PackagesPage::default());
    harness.run();
    let built = harness.state().page.view.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.view.builds(), built);

    "gi".clone_into(&mut harness.state_mut().page.search);
    harness.run();
    assert_eq!(harness.state().page.view.builds(), built + 1);

    harness.state_mut().fixture.status_revision += 1;
    harness.run();
    assert_eq!(harness.state().page.view.builds(), built + 2);
}
