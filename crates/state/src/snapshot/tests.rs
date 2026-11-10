use super::{Part, Snapshot};

#[derive(Debug, Default, PartialEq, Eq)]
struct Kernels(Vec<String>);

#[derive(Debug, Default, PartialEq, Eq)]
struct Locales(Vec<String>);

impl Part for Kernels {}

impl Part for Locales {}

#[test]
fn a_missing_part_is_none() {
    assert_eq!(Snapshot::default().part::<Kernels>(), None);
}

#[test]
fn parts_are_kept_apart_by_type() {
    let mut snapshot = Snapshot::default();

    snapshot.insert(Kernels(vec!["linux-zen".to_owned()]));
    snapshot.insert(Locales(vec!["de_DE.UTF-8".to_owned()]));

    assert_eq!(
        snapshot.part::<Kernels>(),
        Some(&Kernels(vec!["linux-zen".to_owned()]))
    );
    assert_eq!(
        snapshot.part::<Locales>(),
        Some(&Locales(vec!["de_DE.UTF-8".to_owned()]))
    );
}

#[test]
fn a_copy_keeps_the_parts_and_takes_more_alone() {
    let mut probed = Snapshot::default();

    probed.insert(Kernels(vec!["linux-zen".to_owned()]));

    let mut copy = probed.clone();

    copy.insert(Locales(vec!["de_DE.UTF-8".to_owned()]));

    assert_eq!(copy.part::<Kernels>(), probed.part::<Kernels>());
    assert!(std::ptr::eq(
        copy.part::<Kernels>().unwrap(),
        probed.part::<Kernels>().unwrap()
    ));
    assert_eq!(probed.part::<Locales>(), None);
}
