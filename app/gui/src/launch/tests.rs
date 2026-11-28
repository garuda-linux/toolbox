use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use toolbox_system::testing::launch::FixedRunner;

use super::{Launcher as _, SystemLauncher};
use crate::command::{AppCommand, Command, CommandBus};
use crate::testing::{PATIENCE, until};

fn launcher(runtime: &tokio::runtime::Runtime, log_dir: &Path) -> (SystemLauncher, CommandBus) {
    let bus = CommandBus::new();
    let launcher = SystemLauncher::logging_to(
        runtime.handle().clone(),
        bus.sender.clone(),
        egui::Context::default(),
        Some(log_dir.to_owned()),
    );

    (launcher, bus)
}

/// A launcher whose programs are the fake's answers, so no program runs.
fn fake_launcher(
    runtime: &tokio::runtime::Runtime,
    answer: FixedRunner,
) -> (SystemLauncher, CommandBus) {
    let bus = CommandBus::new();
    let launcher = SystemLauncher::with_runner(
        runtime.handle().clone(),
        bus.sender.clone(),
        egui::Context::default(),
        None,
        Arc::new(answer),
    );

    (launcher, bus)
}

/// A tool that exits with an error raises the toast, without a program.
#[test]
fn a_fake_tool_that_exits_with_an_error_raises_a_toast_with_its_name() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (launcher, bus) = fake_launcher(&runtime, FixedRunner::Exit(2));

    launcher.start(Path::new("/usr/bin/garuda-tool"), &["--check"]);

    let text = next_toast(&bus);
    assert!(
        text.contains("garuda-tool") && text.contains("ended with an error"),
        "{text}"
    );
}

/// The files of programs that still write or wrote, below `log_dir`.
fn stderr_files(log_dir: &Path) -> usize {
    std::fs::read_dir(log_dir.join("programs")).map_or(0, Iterator::count)
}

#[test]
fn a_missing_program_raises_a_toast_and_leaves_no_file() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let (launcher, bus) = launcher(&runtime, dir.path());

    launcher.start(Path::new("/nonexistent/garuda-installer"), &[]);

    let text = next_toast(&bus);
    assert!(
        text.contains("garuda-installer") && text.contains("could not be started"),
        "{text}"
    );
    until("a program that never ran keeps a file", || {
        stderr_files(dir.path()) == 0
    });
}

/// No pipe ties the program to the window, so its stderr outlives it.
#[test]
fn a_running_program_writes_its_stderr_into_a_file_that_goes_at_its_end() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let (launcher, _bus) = launcher(&runtime, dir.path());

    launcher.start(Path::new("sh"), &["-c", "echo started >&2; sleep 0.5"]);

    until("the program's stderr file is made", || {
        stderr_files(dir.path()) == 1
    });
    until("the empty file is removed after the end", || {
        stderr_files(dir.path()) == 0
    });
}

#[test]
fn a_tool_that_fails_raises_a_toast_with_its_name() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let (launcher, bus) = launcher(&runtime, dir.path());

    launcher.start(Path::new("ls"), &["/nonexistent/garuda-toolbox"]);

    let text = next_toast(&bus);
    assert!(
        text.contains("ls") && text.contains("ended with an error"),
        "{text}"
    );
}

/// The text of the first toast the launcher sends.
fn next_toast(bus: &CommandBus) -> String {
    let until = Instant::now() + PATIENCE;
    loop {
        if let Some(sent) = bus.receiver.drain().next() {
            let Command::App(AppCommand::Toast { text, kind: _ }) = sent.command else {
                panic!("not a toast: {:?}", sent.command);
            };

            return text;
        }

        assert!(Instant::now() < until, "no toast came");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_tool_that_ends_well_raises_nothing() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let (launcher, bus) = launcher(&runtime, dir.path());

    launcher.start(Path::new("true"), &[]);
    std::thread::sleep(Duration::from_millis(300));

    assert!(bus.receiver.drain().next().is_none());
}

#[test]
fn the_files_an_earlier_start_left_are_removed() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("programs")).unwrap();
    std::fs::write(dir.path().join("programs/tool-1-0.stderr"), "old\n").unwrap();

    let (_launcher, _bus) = launcher(&runtime, dir.path());

    until("the old file is removed", || stderr_files(dir.path()) == 0);
}
