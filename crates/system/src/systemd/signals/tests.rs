use super::RELISTING;

/// A burst of property changes from every running timer must not list
/// again, and a job's start comes with its end.
#[test]
fn only_changes_of_the_units_and_their_files_list_again() {
    for quiet in ["PropertiesChanged", "JobNew", "StartupFinished"] {
        assert!(!RELISTING.contains(&quiet), "{quiet}");
    }

    assert!(RELISTING.contains(&"JobRemoved"));
    assert!(RELISTING.contains(&"UnitFilesChanged"));
}
