use super::TargetList;

#[test]
fn one_target_is_never_empty() {
    let list = TargetList::one("sshd.service");

    assert!(!list.is_empty());
    assert_eq!(list.len(), 1);
    assert_eq!(list.iter().collect::<Vec<_>>(), [&"sshd.service"]);
    assert_eq!((&list).into_iter().collect::<Vec<_>>(), [&"sshd.service"]);
}
