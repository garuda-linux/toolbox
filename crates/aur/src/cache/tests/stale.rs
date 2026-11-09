use super::{DAY, Fake, FakeClock, ONE_NAMES, START, Scripted, cache, names, one};
use crate::AurSource as _;
use crate::transport::Validators;

#[test]
fn a_copy_after_a_failed_refresh_is_stale_until_one_works() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fresh(one(), Validators::default()),
        Scripted::Fails,
        Scripted::NotModified,
    ]);

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);

    assert!(!cache.index().unwrap().stale);

    clock.set(START + DAY);
    let failed = cache.index().unwrap();

    assert!(failed.stale);
    assert_eq!(names(&failed), ONE_NAMES);
    assert!(cache.stored().unwrap().stale, "the mark is kept on disk");

    clock.set(START + 2 * DAY);
    assert!(!cache.index().unwrap().stale);
}

/// An update may change the copy's format. Counted as broken storage, it
/// would hide the AUR until the next day.
#[test]
fn an_unreadable_copy_is_no_copy_and_downloads_at_once() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fresh(one(), Validators::default()),
        Scripted::Fresh(one(), Validators::default()),
    ]);

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);

    cache.index().unwrap();

    // Changed: the stored copy as an older version of the app wrote it.
    std::fs::write(folder.path().join("index.json"), r#"{"old":true}"#).unwrap();
    clock.set(START + 60);

    assert_eq!(names(&cache.index().unwrap()), ONE_NAMES);
    assert_eq!(transport.sent().len(), 2);
    assert_eq!(transport.sent()[1].1, Validators::default());
}
