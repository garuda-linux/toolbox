use std::path::Path;
use std::time::{Duration, Instant};

use super::{Launcher as _, SystemLauncher, TAIL_LINES, Tail};
use crate::command::{AppCommand, Command, CommandBus};
use crate::testing::PATIENCE;

fn launcher(runtime: &tokio::runtime::Runtime) -> (SystemLauncher, CommandBus) {
    let bus = CommandBus::new();
    let launcher = SystemLauncher::new(
        runtime.handle().clone(),
        bus.sender.clone(),
        egui::Context::default(),
    );

    (launcher, bus)
}

#[test]
fn a_missing_program_does_not_start() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (launcher, _bus) = launcher(&runtime);

    let started = launcher.start(Path::new("/nonexistent/garuda-installer"), &[]);

    assert_eq!(started.unwrap_err().kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn a_tool_that_fails_raises_a_toast_with_its_name() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (launcher, bus) = launcher(&runtime);

    launcher
        .start(Path::new("ls"), &["/nonexistent/garuda-toolbox"])
        .unwrap();

    let until = Instant::now() + PATIENCE;
    let text = loop {
        if let Some(sent) = bus.receiver.drain().next() {
            let Command::App(AppCommand::Toast { text, kind: _ }) = sent.command else {
                panic!("not a toast: {:?}", sent.command);
            };

            break text;
        }

        assert!(Instant::now() < until, "no toast came");
        std::thread::sleep(Duration::from_millis(20));
    };

    assert!(
        text.contains("ls") && text.contains("ended with an error"),
        "{text}"
    );
}

#[test]
fn a_tool_that_ends_well_raises_nothing() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let (launcher, bus) = launcher(&runtime);

    launcher.start(Path::new("true"), &[]).unwrap();
    std::thread::sleep(Duration::from_millis(300));

    assert!(bus.receiver.drain().next().is_none());
}

#[test]
fn the_tail_keeps_only_the_last_lines() {
    let mut tail = Tail::default();
    for number in 0..TAIL_LINES + 2 {
        tail.push(format!("line {number}"));
    }

    let text = tail.text();
    assert!(!text.contains("line 1\n"), "{text}");
    assert!(text.starts_with("line 2"), "{text}");
    assert!(
        text.ends_with(&format!("line {}", TAIL_LINES + 1)),
        "{text}"
    );
}
