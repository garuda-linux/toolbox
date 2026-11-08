use std::fs::{self, File};
use std::io::Write as _;
use std::os::fd::OwnedFd;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::Duration;

use rustix::event::{PollFlags, Timespec};
use tempfile::TempDir;

use super::{Chunk, pump, take_text, wait_ready};

fn output(content: &[u8]) -> (TempDir, File) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("output");
    fs::write(&path, content).unwrap();
    let file = File::open(&path).unwrap();

    (dir, file)
}

fn channel() -> (SyncSender<Chunk>, Receiver<Chunk>) {
    sync_channel(4)
}

#[test]
fn complete_text_is_taken_whole() {
    let mut pending = b"plain ascii".to_vec();
    assert_eq!(take_text(&mut pending), "plain ascii");
    assert_eq!(pending, b"");
}

#[test]
fn a_character_split_across_reads_waits_for_its_rest() {
    let euro = "€".as_bytes();
    let mut pending = b"price: ".to_vec();
    pending.extend_from_slice(&euro[..2]);

    assert_eq!(take_text(&mut pending), "price: ");
    assert_eq!(pending, &euro[..2]);

    pending.extend_from_slice(&euro[2..]);
    assert_eq!(take_text(&mut pending), "€");
    assert_eq!(pending, b"");
}

#[test]
fn invalid_bytes_become_replacement_characters() {
    let mut pending = b"a\xffb".to_vec();
    assert_eq!(take_text(&mut pending), "a\u{fffd}b");
    assert_eq!(pending, b"");
}

#[test]
fn a_partial_character_behind_invalid_bytes_waits_for_its_rest() {
    let mut pending = b"a\xff".to_vec();
    pending.extend_from_slice(&"€".as_bytes()[..2]);

    assert_eq!(take_text(&mut pending), "a\u{fffd}");
    assert_eq!(pending, &"€".as_bytes()[..2]);

    pending.extend_from_slice(&"€".as_bytes()[2..]);
    assert_eq!(take_text(&mut pending), "€");
    assert_eq!(pending, b"");
}

#[test]
fn an_end_of_file_closes_the_stream() {
    let (_dir, file) = output(b"hi");
    let (sender, receiver) = channel();
    pump(file, &sender);

    assert_eq!(
        [receiver.recv().unwrap(), receiver.recv().unwrap()],
        [Chunk::Text("hi".to_owned()), Chunk::Closed]
    );
}

#[test]
fn a_dropped_receiver_ends_the_pump() {
    // The write end stays open, so no end of file can end the pump.
    let (read_end, mut write_end) = std::io::pipe().unwrap();
    write_end.write_all(b"hi").unwrap();
    let file = File::from(OwnedFd::from(read_end));

    let (sender, receiver) = channel();
    drop(receiver);
    let (done, ended) = sync_channel(1);
    std::thread::Builder::new()
        .name("pump under test".to_owned())
        .spawn(move || {
            pump(file, &sender);
            done.send(()).ok();
        })
        .unwrap();

    assert_eq!(ended.recv_timeout(Duration::from_secs(10)), Ok(()));
    drop(write_end);
}

#[test]
fn leftover_bytes_are_flushed_at_the_close() {
    let (_dir, file) = output(b"a\xc3");
    let (sender, receiver) = channel();
    pump(file, &sender);

    assert_eq!(
        [
            receiver.recv().unwrap(),
            receiver.recv().unwrap(),
            receiver.recv().unwrap()
        ],
        [
            Chunk::Text("a".to_owned()),
            Chunk::Text("\u{fffd}".to_owned()),
            Chunk::Closed
        ]
    );
}

#[test]
fn an_unpollable_terminal_ends_the_wait() {
    // A negative timeout is no wait at all: the poll refuses it.
    let never = Timespec {
        tv_sec: -1,
        tv_nsec: 0,
    };

    let terminal = File::open("/dev/null").unwrap();
    wait_ready(&terminal, PollFlags::IN, Some(&never)).unwrap_err();
}
