use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

use parking_lot::Mutex;

use super::{Cache, Clock};
use crate::{NewsError, NewsSource as _, REFRESH_EVERY, RequestError, Source, Transport};

/// The forum's announcements feed, newest post first.
const FEED: &str = include_str!("../feed/fixtures/garuda-announcements.rss");
const NEWEST: &str = "Chaotic-AUR turns 8 years old today! 🎉";

struct FakeClock(AtomicI64);

impl Clock for FakeClock {
    fn now(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

struct FakeTransport {
    answers: Mutex<Vec<Result<Vec<u8>, RequestError>>>,
    asked: AtomicUsize,
}

impl Transport for FakeTransport {
    fn get(&self, _url: &str) -> Result<Vec<u8>, RequestError> {
        self.asked.fetch_add(1, Ordering::SeqCst);
        self.answers.lock().remove(0)
    }
}

fn hours(count: i64) -> i64 {
    count * 3_600
}

fn setup(
    answers: Vec<Result<Vec<u8>, RequestError>>,
) -> (tempfile::TempDir, Arc<FakeClock>, Arc<FakeTransport>, Cache) {
    let folder = tempfile::tempdir().unwrap();
    let clock = Arc::new(FakeClock(AtomicI64::new(1_000_000)));
    let transport = Arc::new(FakeTransport {
        answers: Mutex::new(answers),
        asked: AtomicUsize::new(0),
    });

    let cache = Cache::new(
        folder.path().join("news"),
        Source::Garuda,
        Arc::<FakeTransport>::clone(&transport),
        Arc::<FakeClock>::clone(&clock),
    );

    (folder, clock, transport, cache)
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "an answer the fake transport hands out"
)]
fn feed() -> Result<Vec<u8>, RequestError> {
    Ok(FEED.as_bytes().to_vec())
}

fn offline() -> Result<Vec<u8>, RequestError> {
    Err(RequestError::NoAnswer("no network".to_owned()))
}

#[test]
fn the_first_read_downloads_and_keeps_the_posts() {
    let (_folder, _clock, transport, cache) = setup(vec![feed()]);

    let news = cache.news().unwrap();

    assert_eq!(news.posts.len(), crate::feed::MAX_POSTS);
    assert_eq!(news.posts[0].title, NEWEST);
    assert_eq!(news.downloaded, 1_000_000);
    assert_eq!(transport.asked.load(Ordering::SeqCst), 1);
}

#[test]
fn a_second_read_within_six_hours_asks_nobody() {
    let (_folder, clock, transport, cache) = setup(vec![feed()]);
    cache.news().unwrap();

    clock.0.fetch_add(hours(5), Ordering::SeqCst);
    let news = cache.news().unwrap();

    assert_eq!(news.posts.len(), crate::feed::MAX_POSTS);
    assert_eq!(transport.asked.load(Ordering::SeqCst), 1);
    assert_eq!(REFRESH_EVERY.as_secs(), 6 * 3_600);
}

#[test]
fn offline_after_six_hours_the_copy_still_answers() {
    let (_folder, clock, transport, cache) = setup(vec![feed(), offline()]);
    cache.news().unwrap();

    clock.0.fetch_add(hours(7), Ordering::SeqCst);
    let news = cache.news().unwrap();

    assert_eq!(news.posts[0].title, NEWEST);
    assert_eq!(news.downloaded, 1_000_000, "the old copy's time");
    assert_eq!(transport.asked.load(Ordering::SeqCst), 2);
}

#[test]
fn offline_without_a_copy_says_why_then_waits() {
    let (_folder, clock, transport, cache) = setup(vec![offline()]);

    assert_eq!(
        cache.news(),
        Err(NewsError::Download("no network".to_owned()))
    );

    clock.0.fetch_add(hours(1), Ordering::SeqCst);
    assert!(matches!(cache.news(), Err(NewsError::Waiting { next: _ })));
    assert_eq!(transport.asked.load(Ordering::SeqCst), 1);
}

#[test]
fn a_broken_feed_keeps_the_old_copy() {
    let broken = Ok(b"<rss><channel><item>".to_vec());
    let (_folder, clock, _transport, cache) = setup(vec![feed(), broken]);
    cache.news().unwrap();

    clock.0.fetch_add(hours(7), Ordering::SeqCst);

    assert_eq!(cache.news().unwrap().posts[0].title, NEWEST);
}

#[test]
fn a_clock_set_back_waits_from_the_present() {
    let (_folder, clock, transport, cache) = setup(vec![feed(), feed()]);
    cache.news().unwrap();

    clock.0.fetch_sub(hours(100), Ordering::SeqCst);
    cache.news().unwrap();
    assert_eq!(transport.asked.load(Ordering::SeqCst), 1);

    clock.0.fetch_add(hours(7), Ordering::SeqCst);
    cache.news().unwrap();
    assert_eq!(transport.asked.load(Ordering::SeqCst), 2);
}
