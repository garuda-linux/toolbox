use super::RemoteAction;

#[test]
fn every_remote_action_has_its_own_id() {
    let mut ids: Vec<&str> = RemoteAction::ALL
        .into_iter()
        .map(RemoteAction::as_str)
        .collect();

    ids.sort_unstable();
    ids.dedup();

    assert_eq!(ids.len(), RemoteAction::ALL.len());
}
