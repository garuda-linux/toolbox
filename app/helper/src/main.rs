//! `garuda-toolbox-helper`: runs the app's root work, started through
//! `pkexec`.
//!
//! ```text
//! garuda-toolbox-helper apply         # request and answers on stdin, events on stdout
//! garuda-toolbox-helper read <kind>   # one JSON line on stdout
//! garuda-toolbox-helper logs <kind>   # the same, for the kernel log and the journal
//! garuda-toolbox-helper fetch <path>  # the text of one config file as a JSON line
//! ```
//!
//! The request is a typed plan from `toolbox-plan`. Names inside it are
//! validated while it is parsed and every command runs from a fixed path
//! with a clean environment, so nothing the caller sends becomes a shell
//! command or an option.

mod fetch;
mod io;
mod logger;
mod read;
#[cfg(test)]
mod testing;
#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::io::BufReader;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use rustix::process::{DumpableBehavior, Resource, Rlimit};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use toolbox_exec::{Environment, EventSink as _, PtyRunner, apply, preflight};
use toolbox_plan::protocol::{Event, Mode};

use crate::io::{StdoutSink, note, read_request, read_secrets, refuse, spawn_answer_reader};

const USAGE: &str = "usage: garuda-toolbox-helper apply | read <kind> | logs <kind> | fetch <path> (the app starts it, not a person)";

fn main() -> ExitCode {
    // pkexec keeps the caller's umask. Files written as root must not
    // become writable by others.
    rustix::process::umask(rustix::fs::Mode::from_raw_mode(0o022));
    logger::init();
    keep_memory_private();

    // `std::env::args` panics on an argument that is no UTF-8.
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let Some(args) = args
        .iter()
        .map(|arg| arg.to_str())
        .collect::<Option<Vec<_>>>()
    else {
        return refuse(USAGE);
    };

    match args.as_slice() {
        ["apply"] if rustix::termios::isatty(rustix::stdio::stdin()) => {
            refuse("it reads a request from the app, not from a terminal")
        }
        ["apply"] => run(),
        [command @ ("read" | "logs"), kind] => read::run(command, kind),
        ["fetch", path] => fetch::run(path),
        _ => refuse(USAGE),
    }
}

/// The helper holds passwords and passphrases: no core file, and no
/// other process of the user may read its memory. Best effort, a kernel
/// that refuses changes nothing else.
fn keep_memory_private() {
    let none = Rlimit {
        current: Some(0),
        maximum: Some(0),
    };

    if let Err(error) = rustix::process::setrlimit(Resource::Core, none) {
        log::warn!("cannot switch core files off: {error}");
    }

    if let Err(error) = rustix::process::set_dumpable_behavior(DumpableBehavior::NotDumpable) {
        log::warn!("cannot keep the memory from being read: {error}");
    }
}

fn run() -> ExitCode {
    let mut input = BufReader::new(std::io::stdin());
    let mut sink = StdoutSink;

    let mut request = match read_request(&mut input) {
        Ok(request) => request,
        Err(error) => return reject(&mut sink, &error.to_string()),
    };

    if request.mode == Mode::Run && !rustix::process::geteuid().is_root() {
        return reject(
            &mut sink,
            "root rights are missing (start it through pkexec)",
        );
    }

    // Only after the root check, because without root nothing runs and
    // no password should be taken in.
    if let Err(error) = read_secrets(&mut input, &mut request) {
        return reject(&mut sink, &error.to_string());
    }

    // The names in a boot repair are checked against the real devices
    // before the first step, so a request for a wrong or used disk runs
    // nothing at all. A dry run has no devices to look at.
    if request.mode == Mode::Run
        && let Err(reason) = preflight(request.steps.iter().map(|step| &step.action))
    {
        return reject(&mut sink, &reason.to_string());
    }

    let mut runner =
        PtyRunner::new(spawn_answer_reader(input), Environment::Clean).stop_on(stop_on_signals());

    if apply(request.mode, &request.steps, &mut runner, &mut sink) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// SIGTERM and SIGHUP cancel instead of ending the helper, since their default ends it between a mount and its unmount or with a LUKS container open.
/// SIGINT ends nothing, see `docs/agents/security.md`.
fn stop_on_signals() -> Arc<AtomicBool> {
    // SIGINT is a terminal's Ctrl+C. The command line gets it too and
    // answers with a finish or a cancel, so its flag is never read.
    let stop = Arc::new(AtomicBool::new(false));

    for signal in [SIGTERM, SIGHUP] {
        if let Err(error) = signal_hook::flag::register(signal, Arc::clone(&stop)) {
            log::warn!("signal {signal} will end the helper at once: {error}");
        }
    }

    if let Err(error) = signal_hook::flag::register(SIGINT, Arc::new(AtomicBool::new(false))) {
        log::warn!("Ctrl+C in a terminal will end the helper at once: {error}");
    }

    stop
}

fn reject(sink: &mut StdoutSink, reason: &str) -> ExitCode {
    note(reason);
    sink.send(Event::Rejected {
        reason: reason.to_owned(),
    });
    sink.send(Event::Done);
    ExitCode::FAILURE
}
