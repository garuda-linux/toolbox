use std::sync::Arc;

use toolbox_gui_kit::Loadable;
use toolbox_services::home::StatusRead;
use toolbox_system::Status;

use crate::pages::home::HomeFacts;

fn quiet_status() -> Status {
    Status {
        updates: Ok(Vec::new()),
        aur_updates: Ok(Vec::new()),
        merges: Ok(Vec::new()),
        last_update: Ok(None),
        health: Ok(Vec::new()),
    }
}

fn facts(status: Option<&Loadable<Status>>) -> HomeFacts<'_> {
    HomeFacts {
        status,
        status_revision: 0,
        facts: None,
        news: None,
        programs: None,
        programs_failed: false,
        problem: None,
        recent: &[],
        setup_finished: false,
    }
}

#[test]
fn a_failed_status_read_stays_failed_beside_an_older_status() {
    let failed = Loadable::Failed {
        error: "no answer".to_owned(),
        previous: Some(Arc::new(quiet_status())),
    };

    let loading = Loadable::Loading {
        previous: Some(Arc::new(quiet_status())),
    };

    assert!(matches!(facts(None).status_read(), StatusRead::Unread));
    assert!(matches!(
        facts(Some(&failed)).status_read(),
        StatusRead::Failed
    ));
    assert!(matches!(
        facts(Some(&loading)).status_read(),
        StatusRead::Read(_)
    ));
}

#[test]
fn a_status_read_that_is_loading_without_an_older_status_is_unread() {
    let loading = Loadable::<Status>::Loading { previous: None };

    assert!(matches!(
        facts(Some(&loading)).status_read(),
        StatusRead::Unread
    ));
}
