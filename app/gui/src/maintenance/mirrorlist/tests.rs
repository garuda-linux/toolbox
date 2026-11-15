use toolbox_plan::{Mirrorlist, Plan, RootAction};

use super::Install;

fn plan() -> Plan {
    let mirrorlist =
        Mirrorlist::new("Server = https://cdn-mirror.chaotic.cx/$repo/$arch\n").unwrap();

    Plan::root(RootAction::InstallMirrorlist { mirrorlist })
}

#[test]
fn a_list_that_started_waits_for_nothing() {
    let mut install = Install::default();
    let sent = install.landed(plan());

    assert_eq!(sent, plan());
    assert_eq!(install.clone().started(), Install::None);
}

#[test]
fn a_refused_list_goes_out_again_after_the_end_of_the_plan_that_ran() {
    let mut install = Install::default();
    install.landed(plan());
    install = install.refused();

    assert_eq!(install.due(), None, "the other plan still runs");

    install = install.finished();

    assert_eq!(install.due(), Some(plan()));
    assert_eq!(install, Install::Sent(plan()));
    assert_eq!(install.due(), None, "sent once");
}

#[test]
fn the_end_of_another_plan_leaves_a_sent_list_alone() {
    let mut install = Install::default();
    install.landed(plan());

    install = install.finished();

    assert_eq!(install, Install::Sent(plan()));
}
