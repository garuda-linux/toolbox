mod stored;

use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use parking_lot::Mutex;

use super::{Cache, Clock};
use crate::dump::tests::{DUMP, gzipped, unpacked};
use crate::transport::{Answer, Transport, Validators};
use crate::{AurError, AurSource as _, DUMP_URL, RequestError};

const DAY: i64 = 24 * 60 * 60;
const START: i64 = 1_700_000_000;
const ONE_NAMES: [(&str, &str); 5] = [
    ("0ad-boongui", "2.6.11-2"),
    ("0c-ime", "0.2.9-1"),
    ("0xtools", "2.0.3-1"),
    ("10ff-git", "r56.9be5cf0-1"),
    ("10mb.video-git", "r5.940b95e-1"),
];
const TWO_NAMES: [(&str, &str); 4] = [
    ("0ad-boongui", "2.6.11-2"),
    ("0c-ime", "0.3.0-1"),
    ("0xtools", "2.0.3-1"),
    ("10ff-git", "r56.9be5cf0-1"),
];

#[derive(Clone)]
enum Scripted {
    Fresh(Vec<u8>, Validators),
    NotModified,
    Fails,
}

struct Fake {
    script: Mutex<Vec<Scripted>>,
    sent: Mutex<Vec<(String, Validators)>>,
    slow: bool,
}

impl Fake {
    fn new(script: impl IntoIterator<Item = Scripted>) -> Arc<Self> {
        let mut script: Vec<Scripted> = script.into_iter().collect();
        script.reverse();
        Arc::new(Self {
            script: Mutex::new(script),
            sent: Mutex::new(Vec::new()),
            slow: false,
        })
    }

    fn sent(&self) -> Vec<(String, Validators)> {
        self.sent.lock().clone()
    }
}

impl Transport for Fake {
    fn get(&self, url: &str, validators: &Validators) -> Result<Answer, RequestError> {
        self.sent.lock().push((url.to_owned(), validators.clone()));
        if self.slow {
            std::thread::sleep(Duration::from_millis(50));
        }

        let next = self.script.lock().pop();
        match next {
            Some(Scripted::Fresh(dump, answered)) => Ok(Answer::Fresh {
                body: Box::new(Cursor::new(dump)),
                validators: answered,
            }),
            Some(Scripted::NotModified) => Ok(Answer::NotModified),
            Some(Scripted::Fails) | None => Err(RequestError::NoAnswer("offline".to_owned())),
        }
    }
}

struct FakeClock(AtomicI64);

impl FakeClock {
    fn at(seconds: i64) -> Arc<Self> {
        Arc::new(Self(AtomicI64::new(seconds)))
    }

    fn set(&self, seconds: i64) {
        self.0.store(seconds, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

fn tagged(etag: &str, date: &str) -> Validators {
    Validators {
        etag: Some(etag.to_owned()),
        last_modified: Some(date.to_owned()),
    }
}

fn cache(folder: &Path, transport: &Arc<Fake>, clock: &Arc<FakeClock>) -> Cache {
    Cache::new(
        folder.to_path_buf(),
        Arc::<Fake>::clone(transport),
        Arc::<FakeClock>::clone(clock),
    )
}

fn one() -> Vec<u8> {
    DUMP.to_vec()
}

/// The dump of a later day, edited from [`DUMP`]: `0c-ime` at a new
/// version and `10mb.video-git` gone.
fn two() -> Vec<u8> {
    let bumped = unpacked().replacen(r#""Version":"0.2.9-1""#, r#""Version":"0.3.0-1""#, 1);
    let mut entries: Vec<serde_json::Value> = serde_json::from_str(&bumped).unwrap();

    entries.pop();
    gzipped(&serde_json::to_string(&entries).unwrap())
}

fn names(index: &toolbox_state::aur::AurIndex) -> Vec<(&str, &str)> {
    index
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package.version.as_str()))
        .collect()
}

#[test]
fn the_first_call_downloads_the_dump_without_validators() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([Scripted::Fresh(one(), tagged("\"a\"", "Mon"))]);
    let clock = FakeClock::at(START);

    let index = cache(folder.path(), &transport, &clock).index().unwrap();

    assert_eq!(names(&index), ONE_NAMES);
    assert_eq!(index.downloaded, START);
    assert_eq!(
        transport.sent(),
        [(DUMP_URL.to_owned(), Validators::default())]
    );
}

#[test]
fn a_second_call_within_a_day_sends_nothing() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([Scripted::Fresh(one(), Validators::default())]);
    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);
    cache.index().unwrap();

    clock.set(START + DAY - 1);
    let index = cache.index().unwrap();

    assert_eq!(names(&index), ONE_NAMES);
    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn the_rule_survives_a_new_cache_on_the_same_folder() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([Scripted::Fresh(one(), Validators::default())]);
    let clock = FakeClock::at(START);
    cache(folder.path(), &transport, &clock).index().unwrap();

    cache(folder.path(), &transport, &clock).index().unwrap();

    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn a_failure_also_waits_a_day() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fails,
        Scripted::Fresh(one(), Validators::default()),
    ]);

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);

    assert_eq!(cache.index(), Err(AurError::Download("offline".to_owned())));

    clock.set(START + DAY - 1);
    assert_eq!(cache.index(), Err(AurError::Waiting { next: START + DAY }));
    assert_eq!(transport.sent().len(), 1);

    clock.set(START + DAY);
    assert_eq!(names(&cache.index().unwrap()), ONE_NAMES);
    assert_eq!(transport.sent().len(), 2);
}

#[test]
fn after_a_day_the_request_carries_the_stored_validators() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fresh(one(), tagged("\"abc\"", "Tue, 01 Oct 2024 00:00:00 GMT")),
        Scripted::Fresh(two(), tagged("\"def\"", "Wed, 02 Oct 2024 00:00:00 GMT")),
    ]);

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);
    cache.index().unwrap();

    clock.set(START + DAY);
    let index = cache.index().unwrap();

    assert_eq!(names(&index), TWO_NAMES);
    assert_eq!(index.downloaded, START + DAY);
    let sent = transport.sent();
    assert_eq!(
        sent.get(1),
        Some(&(
            DUMP_URL.to_owned(),
            tagged("\"abc\"", "Tue, 01 Oct 2024 00:00:00 GMT")
        ))
    );
}

#[test]
fn not_modified_keeps_the_copy_and_counts_as_the_attempt() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fresh(one(), tagged("\"a\"", "Mon")),
        Scripted::NotModified,
    ]);

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);
    cache.index().unwrap();

    clock.set(START + DAY);
    let index = cache.index().unwrap();
    clock.set(START + 2 * DAY - 1);
    cache.index().unwrap();

    assert_eq!(names(&index), ONE_NAMES);
    assert_eq!(index.downloaded, START);
    assert_eq!(transport.sent().len(), 2);
}

#[test]
fn an_older_copy_answers_after_a_failed_refresh() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fresh(one(), Validators::default()),
        Scripted::Fails,
    ]);

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);
    cache.index().unwrap();

    clock.set(START + DAY);

    assert_eq!(names(&cache.index().unwrap()), ONE_NAMES);
    assert_eq!(transport.sent().len(), 2);
}

#[test]
fn an_unreadable_dump_is_a_failed_attempt() {
    let folder = tempfile::tempdir().unwrap();

    // The dump cut in half, as a broken download leaves it.
    let cut = DUMP.get(..DUMP.len() / 2).unwrap().to_vec();
    let transport = Fake::new([Scripted::Fresh(cut, Validators::default())]);
    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);

    assert!(matches!(cache.index(), Err(AurError::Download(_))));
    assert_eq!(cache.index(), Err(AurError::Waiting { next: START + DAY }));
    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn a_last_attempt_in_the_future_counts_as_now() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fresh(one(), Validators::default()),
        Scripted::NotModified,
    ]);

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);
    cache.index().unwrap();

    let back = START - 10 * DAY;
    clock.set(back);
    cache.index().unwrap();
    assert_eq!(
        transport.sent().len(),
        1,
        "sent at once after the clock moved back"
    );

    clock.set(back + DAY - 1);
    cache.index().unwrap();
    assert_eq!(
        transport.sent().len(),
        1,
        "sent before a day from the present"
    );

    clock.set(back + DAY);
    cache.index().unwrap();
    assert_eq!(transport.sent().len(), 2, "waited longer than a day");
}

#[test]
fn without_a_copy_a_future_attempt_waits_a_day_from_now() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([Scripted::Fails]);
    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);
    cache.index().unwrap_err();

    let back = START - 10 * DAY;
    clock.set(back);

    assert_eq!(
        cache.index().unwrap_err(),
        AurError::Waiting { next: back + DAY }
    );
    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn without_a_copy_the_request_carries_no_validators() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fresh(one(), tagged("\"a\"", "Mon")),
        Scripted::Fresh(one(), Validators::default()),
    ]);

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);
    cache.index().unwrap();
    std::fs::remove_file(folder.path().join("index.json")).unwrap();

    clock.set(START + DAY);
    cache.index().unwrap();

    assert_eq!(
        transport.sent().get(1).map(|sent| &sent.1),
        Some(&Validators::default())
    );
}

#[test]
fn two_callers_at_once_make_one_request() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Arc::new(Fake {
        slow: true,
        ..Arc::into_inner(Fake::new([Scripted::Fresh(one(), Validators::default())])).unwrap()
    });

    let clock = FakeClock::at(START);
    let cache = cache(folder.path(), &transport, &clock);

    std::thread::scope(|scope| {
        let first = scope.spawn(|| cache.index());
        let second = scope.spawn(|| cache.index());
        first.join().unwrap().unwrap();
        second.join().unwrap().unwrap();
    });

    assert_eq!(transport.sent().len(), 1);
}

#[test]
fn the_files_are_private() {
    use std::os::unix::fs::PermissionsExt as _;

    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([Scripted::Fresh(one(), Validators::default())]);
    let clock = FakeClock::at(START);
    cache(folder.path(), &transport, &clock).index().unwrap();

    for name in ["index.json", "meta.json"] {
        let mode = std::fs::metadata(folder.path().join(name))
            .unwrap()
            .permissions()
            .mode();

        assert_eq!(mode & 0o777, 0o600, "{name}");
    }
}
