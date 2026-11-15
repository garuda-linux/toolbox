use std::io::Write as _;

use super::{StderrFile, sweep};

#[test]
fn the_end_reads_the_last_lines_and_removes_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let (output, mut handle) = StderrFile::create(dir.path(), "ls").unwrap();
    for number in 0..7_u32 {
        writeln!(handle, "line {number}").unwrap();
    }

    let path = output.path.clone();
    let lines = output.finish(5);

    assert_eq!(lines, ["line 2", "line 3", "line 4", "line 5", "line 6"]);
    assert!(!path.exists());
}

/// A non UTF-8 byte once ended the pipe's reader and the program with it.
#[test]
fn bytes_that_are_no_utf8_are_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let (output, mut handle) = StderrFile::create(dir.path(), "tool").unwrap();
    handle.write_all(b"bad \xff byte\nlast\n").unwrap();

    assert_eq!(output.finish(5), ["bad \u{fffd} byte", "last"]);
}

#[test]
fn two_programs_of_one_name_get_files_of_their_own() {
    let dir = tempfile::tempdir().unwrap();
    let (first, _first_handle) = StderrFile::create(dir.path(), "tool").unwrap();
    let (second, _second_handle) = StderrFile::create(dir.path(), "tool").unwrap();

    assert_ne!(first.path, second.path);
}

/// Process 1 is init, so its name stands for an earlier start.
#[test]
fn the_sweep_removes_only_the_files_programs_of_earlier_starts_left() {
    let dir = tempfile::tempdir().unwrap();
    let (own, _handle) = StderrFile::create(dir.path(), "tool").unwrap();
    let left = dir.path().join("programs/1-tool-0-0.stderr");
    std::fs::write(&left, "old\n").unwrap();
    let log = dir.path().join("garuda-toolbox.log");
    std::fs::write(&log, "kept\n").unwrap();

    sweep(dir.path());

    assert!(!left.exists());
    assert!(own.path.exists(), "this start's program still writes");
    assert!(log.exists());
}
