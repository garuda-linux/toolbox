use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;

use super::super::super::PackagesPage;
use super::super::super::tests::{catalog, gimp_details};
use super::{change, gimp_with, showing};
use crate::packages::{Details, Packages, Read};
use crate::testing::PageState;

fn with_history(days: &[u8]) -> Details {
    Details {
        history: Read::Known(days.iter().copied().map(change).collect()),
        ..gimp_details()
    }
}

/// gimp with three files, its list unfolded.
fn unfolded_files() -> Harness<'static, PageState<PackagesPage>> {
    let mut harness = showing(gimp_with(|info| {
        if let Some(installed) = &mut info.installed {
            installed.files = ["/usr/bin/gimp", "/usr/share/gimp/", "/usr/share/man/gimp.1"]
                .map(str::to_owned)
                .to_vec();
        }
    }));

    harness.state_mut().page.unfolded.files = true;
    harness.run();
    harness
}

#[test]
fn the_filter_keeps_the_files_that_hold_its_text() {
    let mut harness = unfolded_files();
    assert!(harness.query_by_label("/usr/bin/gimp").is_some());

    "share".clone_into(&mut harness.state_mut().page.file_filter);
    harness.run();

    assert!(harness.query_by_label("/usr/bin/gimp").is_none());
    assert!(harness.query_by_label("/usr/share/gimp/").is_some());
    assert!(harness.query_by_label("/usr/share/man/gimp.1").is_some());
}

#[test]
fn a_filter_no_file_holds_says_so() {
    let mut harness = unfolded_files();

    "kate".clone_into(&mut harness.state_mut().page.file_filter);
    harness.run();

    assert!(harness.query_by_label("No file matches “kate”.").is_some());
}

#[test]
fn the_filter_is_a_named_field() {
    let harness = unfolded_files();

    assert!(
        harness
            .query_by_role_and_label(Role::TextInput, "Filter files")
            .is_some()
    );
}

#[test]
fn a_frame_with_the_same_history_builds_no_history_lines() {
    let mut harness = showing(with_history(&[1, 2, 3]));
    let built = harness.state().page.history.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.history.builds(), built);

    harness.state_mut().page.unfolded.history_all = true;
    harness.run();
    assert_eq!(harness.state().page.history.builds(), built + 1);

    harness.state_mut().fixture.services.packages =
        Packages::showing(Read::Known(catalog()), Some(with_history(&[4])));
    harness.run();
    assert_eq!(harness.state().page.history.builds(), built + 2);
}

#[test]
fn a_frame_with_the_same_details_builds_no_facts() {
    let mut harness = showing(with_history(&[1]));
    let built = harness.state().page.facts.builds();

    harness.step();
    harness.step();
    assert_eq!(harness.state().page.facts.builds(), built);

    harness.state_mut().fixture.services.packages =
        Packages::showing(Read::Known(catalog()), Some(with_history(&[2])));
    harness.run();
    assert_eq!(harness.state().page.facts.builds(), built + 1);
}
