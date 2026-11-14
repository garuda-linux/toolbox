use toolbox_gui_kit::widgets::{ListItem as _, Rows as _};

use super::super::super::tests::{catalog, gimp_details};
use super::super::below::Files;
use super::{change, showing};
use crate::packages::{Details, Packages, Read};

fn with_history(days: &[u8]) -> Details {
    Details {
        history: Read::Known(days.iter().copied().map(change).collect()),
        ..gimp_details()
    }
}

#[test]
fn the_files_are_lines_only_when_the_list_asks_for_them() {
    let files = ["/usr/bin/gimp".to_owned(), "/usr/lib/gimp".to_owned()];
    let shown = Files(&files);

    assert_eq!(shown.len(), 2);
    assert_eq!(
        shown.get(1).map(|line| line.key().to_owned()).as_deref(),
        Some("/usr/lib/gimp")
    );
    assert!(shown.get(2).is_none());
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
