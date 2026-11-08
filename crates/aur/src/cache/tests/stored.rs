use super::{DAY, FakeClock, ONE, START, Scripted, TWO, cache, names};
use crate::transport::Validators;
use crate::{AurError, AurSource as _};

#[test]
fn a_copy_answers_without_a_request_even_when_a_day_old() {
    let folder = tempfile::tempdir().unwrap();
    let transport = super::Fake::new([
        Scripted::Fresh(ONE, Validators::default()),
        Scripted::Fresh(TWO, Validators::default()),
    ]);

    let clock = FakeClock::at(START);
    let aur = cache(folder.path(), &transport, &clock);
    aur.index().unwrap();
    clock.set(START + 2 * DAY);

    let index = aur.stored().unwrap();

    assert_eq!(names(&index), [("paru", "1-1")]);
    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn without_a_copy_it_downloads_one() {
    let folder = tempfile::tempdir().unwrap();
    let transport = super::Fake::new([Scripted::Fresh(ONE, Validators::default())]);
    let clock = FakeClock::at(START);

    let index = cache(folder.path(), &transport, &clock).stored().unwrap();

    assert_eq!(names(&index), [("paru", "1-1")]);
    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn without_a_copy_it_keeps_the_once_a_day_rule() {
    let folder = tempfile::tempdir().unwrap();
    let transport =
        super::Fake::new([Scripted::Fails, Scripted::Fresh(ONE, Validators::default())]);

    let clock = FakeClock::at(START);
    let aur = cache(folder.path(), &transport, &clock);
    aur.stored().unwrap_err();

    let waiting = aur.stored();

    assert_eq!(waiting, Err(AurError::Waiting { next: START + DAY }));
    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn the_copy_is_known_by_when_it_was_written() {
    let folder = tempfile::tempdir().unwrap();
    let transport = super::Fake::new([Scripted::Fresh(ONE, Validators::default())]);
    let clock = FakeClock::at(START);
    let aur = cache(folder.path(), &transport, &clock);
    assert_eq!(aur.modified(), None);

    aur.index().unwrap();

    assert!(aur.modified().is_some_and(|stamp| stamp > 0));
}
