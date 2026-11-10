//! `garuda-toolbox status`: the facts read from the system once, or with
//! `--watch` again and again.

use std::io::Write as _;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use tokio::runtime::Handle;
use toolbox_cli::Exit;
use toolbox_cli::args::StatusArgs;
use toolbox_cli::output::json;
use toolbox_system::{
    FactSources, HostSource as _, StatusSources, SystemHost, probe_facts, probe_status,
};

use super::watch::{Shape, Watch, watch};
use super::{StatusView, brief, needs_attention, text, view};
use crate::interrupt::{self, Stopper};
use crate::out::Out;
use crate::spinner;

const LOOK_AGAIN: Duration = Duration::from_millis(100);

pub fn status(args: StatusArgs, runtime: &Handle, out: &mut Out<'_>) -> Exit {
    if args.watch {
        return watched(args, runtime, out);
    }

    let view = read(args.read.json || out.quiet());
    let text = if args.read.json {
        json(&view)
    } else if args.brief {
        format!("{}\n", brief(&view))
    } else {
        text(&view, out.paint(), out.verbose())
    };

    write!(out, "{text}").ok();
    if needs_attention(&view) {
        Exit::Attention
    } else {
        Exit::Success
    }
}

fn watched(args: StatusArgs, runtime: &Handle, out: &mut Out<'_>) -> Exit {
    let shape = if args.read.json {
        Shape::Json
    } else if args.brief {
        Shape::Brief
    } else {
        Shape::Text {
            paint: out.paint(),
            verbose: out.verbose(),
        }
    };

    let quiet = args.read.json || out.quiet();
    let mut stopper = Stopper::new(interrupt::listen(runtime));
    let interval = args.interval.duration();

    watch(
        Watch::new(shape),
        || read(quiet),
        || waited(&mut stopper, interval),
        || jiff::Zoned::now().strftime("%H:%M").to_string(),
        out,
    )
}

/// `quiet` leaves the spinner out.
fn read(quiet: bool) -> StatusView {
    let (found, facts, restart) = spinner::while_reading(quiet, || {
        let found = probe_status(&StatusSources::system());
        let facts = probe_facts(&FactSources::system());
        (found, facts, SystemHost::from_process().is_reboot_pending())
    });

    let now = SystemTime::now();
    view(&found, &restart, now).with_facts(&facts, now)
}

/// Waits `interval`, and tells whether it passed without a Ctrl+C. A
/// Ctrl+C during the read before counts too.
fn waited(stopper: &mut Stopper, interval: Duration) -> bool {
    let until = Instant::now() + interval;

    loop {
        if !stopper.take().is_empty() {
            return true;
        }

        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return false;
        }

        thread::sleep(left.min(LOOK_AGAIN));
    }
}
