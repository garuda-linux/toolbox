#[cfg(test)]
mod tests;

use std::io::{self, Read, Write as _};
use std::process::{Child, ChildStdin};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group};
use toolbox_pty::has_ended;

use super::{CaptureError, Exited, Policy, Tail};

const READ_BYTES: usize = 64 << 10;

/// The longest pause between two looks at the program. The first ones
/// are shorter, so a quick program is not held up.
const LONGEST_PAUSE: Duration = Duration::from_millis(20);

enum Written {
    Done(io::Result<()>),
    Apart(Receiver<io::Result<()>>),
}

struct Watch<'flags> {
    cancel: &'flags AtomicBool,
    overflow: &'flags AtomicBool,
    end: Option<Instant>,
    grace: Option<Duration>,
    group: Pid,
}

pub fn finish(
    mut child: Child,
    policy: &Policy<'_>,
    started: Instant,
    cancel: &Arc<AtomicBool>,
) -> Result<Exited, CaptureError> {
    let overflow = Arc::new(AtomicBool::new(false));
    let watched = policy.cap_is_error.then(|| Arc::clone(&overflow));
    let output = read_tail(child.stdout.take(), policy.limit, cancel, watched);
    let errors = read_tail(child.stderr.take(), policy.error_limit, cancel, None);

    // Inline input fits a pipe, so this write cannot block on a program that prints first.
    let written = match (child.stdin.take(), policy.stdin) {
        (Some(stdin), Some(input)) if policy.apart => Written::Apart(write_apart(stdin, input)),
        (Some(mut stdin), Some(input)) => Written::Done(stdin.write_all(input)),
        _ => Written::Done(Ok(())),
    };

    let watch = Watch {
        cancel,
        overflow: &overflow,
        end: policy.end(started),
        grace: policy.term_grace,
        group: Pid::from_child(&child),
    };

    if let Err(stopped) = wait(&child, &watch) {
        let grace = watch
            .grace
            .filter(|_| matches!(stopped, CaptureError::TimedOut));

        stop(child, grace);
        return Err(stopped);
    }

    // The leader is a zombie now, so the group gets its signals before
    // it is reaped below.
    let collected = collect([&output, &errors], written, policy, &watch);
    let status = child.wait();
    let (printed, complained, truncated) = collected?;

    Ok(Exited {
        status: status.map_err(CaptureError::Failed)?,
        output: printed,
        errors: complained,
        truncated,
    })
}

/// Reads `source` to its end on a thread, so a program that fills one pipe
/// while the other is read never blocks. With `overflow` the read stops
/// at the limit and raises that flag.
fn read_tail<R>(
    source: Option<R>,
    limit: usize,
    cancel: &Arc<AtomicBool>,
    overflow: Option<Arc<AtomicBool>>,
) -> Receiver<Tail>
where
    R: Read + Send + 'static,
{
    let (sender, receiver) = mpsc::channel();
    let cancel = Arc::clone(cancel);
    let spare = sender.clone();

    let reader = thread::Builder::new()
        .name("capture-pipe".to_owned())
        .spawn(move || {
            let mut tail = Tail::new(limit);
            let mut buffer = vec![0; READ_BYTES];
            let mut pipe = source;

            while let Some(open) = pipe.as_mut() {
                match open.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => tail.push(buffer.get(..read).unwrap_or_default()),
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                    Err(error) => {
                        tail.failed = Some(error);
                        break;
                    }
                }

                // Dropping the pipe ends a program that cannot be killed.
                if cancel.load(Ordering::Relaxed) {
                    return;
                }

                if let Some(flag) = overflow.as_ref().filter(|_| tail.truncated) {
                    flag.store(true, Ordering::Relaxed);
                    break;
                }
            }

            // Nobody waits after a cancel.
            sender.send(tail).ok();
        });

    // Without a thread the sender is gone with the closure, and the closed
    // channel would read as a cancel. The spare sender reports the error.
    match reader {
        Ok(_) => drop(spare),
        Err(error) => {
            let mut tail = Tail::new(limit);
            tail.failed = Some(error);
            spare.send(tail).ok();
        }
    }

    receiver
}

/// Left behind like a reader: a stop kills the program, which breaks the write.
fn write_apart(mut stdin: ChildStdin, input: &[u8]) -> Receiver<io::Result<()>> {
    let (sender, receiver) = mpsc::channel();
    let spare = sender.clone();
    let input = input.to_vec();

    let writer = thread::Builder::new()
        .name("capture-stdin".to_owned())
        .spawn(move || {
            let written = stdin.write_all(&input);

            drop(stdin);

            // Nobody waits after a stop.
            sender.send(written).ok();
        });

    // Without a thread the closed channel would read as a cancel. The spare
    // sender reports the error instead.
    match writer {
        Ok(_) => drop(spare),
        Err(error) => {
            spare.send(Err(error)).ok();
        }
    }

    receiver
}

/// Returns once the program ended, leaving it unreaped. A program that
/// ended wins over a cancel, so it is never reported as cancelled.
fn wait(child: &Child, watch: &Watch<'_>) -> Result<(), CaptureError> {
    let mut pause = Duration::from_millis(1);

    loop {
        let stopped = match has_ended(child) {
            Ok(true) => return Ok(()),
            Err(error) => CaptureError::Failed(error),
            Ok(false) if watch.cancel.load(Ordering::Relaxed) => CaptureError::Cancelled,
            Ok(false) if watch.overflow.load(Ordering::Relaxed) => CaptureError::TooLarge,
            Ok(false) if passed(watch.end) => CaptureError::TimedOut,
            Ok(false) => {
                thread::sleep(pause);
                pause = pause.saturating_mul(2).min(LONGEST_PAUSE);
                continue;
            }
        };

        return Err(stopped);
    }
}

/// The checks on the output once the program ended. The group is signalled
/// before the leader is reaped, so no id is reused in between.
fn collect(
    [output, errors]: [&Receiver<Tail>; 2],
    written: Written,
    policy: &Policy<'_>,
    watch: &Watch<'_>,
) -> Result<(Vec<u8>, Vec<u8>, bool), CaptureError> {
    let (group, cancel, end) = (watch.group, watch.cancel, watch.end);
    let written = match written {
        Written::Done(written) => written,
        Written::Apart(writer) => receive(&writer, group, cancel, end)?,
    };

    // A program that exits without reading closes the pipe. Its exit code
    // and its message say why, the broken pipe adds nothing.
    match written {
        Err(error) if error.kind() != io::ErrorKind::BrokenPipe => {
            return Err(CaptureError::Failed(error));
        }
        Ok(()) | Err(_) => {}
    }

    let output = whole(receive(output, group, cancel, end)?)?;
    let errors = whole(receive(errors, group, cancel, end)?)?;
    let (output, truncated) = output.into_bytes();
    let (errors, _) = errors.into_bytes();

    if truncated && policy.cap_is_error {
        return Err(CaptureError::TooLarge);
    }

    Ok((output, errors, truncated))
}

/// Asks the group to end when there is a `grace`, then kills what is
/// left. The leader is still unreaped here, so its group id is reserved
/// while the signals go out. The exit status is collected whenever the
/// program ends, so no zombie stays behind and no caller waits for it.
fn stop(mut child: Child, grace: Option<Duration>) {
    let group = Pid::from_child(&child);

    if let Some(grace) = grace {
        signal(group, Signal::TERM);

        let until = Instant::now().checked_add(grace);

        while matches!(has_ended(&child), Ok(false)) && !passed(until) {
            thread::sleep(LONGEST_PAUSE);
        }
    }

    // Children that ignored the request may still run in the group.
    signal(group, Signal::KILL);

    // Without a thread the killed child stays a zombie until this
    // process ends, which is all that is lost.
    let reaper = thread::Builder::new()
        .name("capture-reap".to_owned())
        .spawn(move || child.wait().ok());

    drop(reaper);
}

/// The tail of one pipe. The program has ended, but what it left running
/// in its `group` may hold the pipe open: the deadline and a cancel end
/// that wait too, kill the group and leave the reader behind.
fn receive<T>(
    tail: &Receiver<T>,
    group: Pid,
    cancel: &AtomicBool,
    end: Option<Instant>,
) -> Result<T, CaptureError> {
    loop {
        let stopped = match tail.recv_timeout(LONGEST_PAUSE) {
            Ok(tail) => return Ok(tail),
            Err(RecvTimeoutError::Disconnected) => return Err(CaptureError::Cancelled),
            Err(RecvTimeoutError::Timeout) if cancel.load(Ordering::Relaxed) => {
                CaptureError::Cancelled
            }
            Err(RecvTimeoutError::Timeout) if passed(end) => CaptureError::TimedOut,
            Err(RecvTimeoutError::Timeout) => continue,
        };

        // The leader is not reaped yet, so the group id is still reserved
        // and this reaches only the leftovers.
        signal(group, Signal::KILL);
        return Err(stopped);
    }
}

fn signal(group: Pid, signal: Signal) {
    match kill_process_group(group, signal) {
        // The whole group is gone already.
        Ok(()) | Err(Errno::SRCH) => {}
        Err(error) => tracing::warn!("the program's group got no {signal:?}: {error}"),
    }
}

fn passed(end: Option<Instant>) -> bool {
    end.is_some_and(|end| Instant::now() >= end)
}

/// A pipe that broke midway leaves a partial output, which a caller such
/// as `pacman -Qtdq` would read as the whole list.
pub(super) fn whole(mut tail: Tail) -> Result<Tail, CaptureError> {
    let failed = tail.failed.take();

    failed.map_or(Ok(tail), |error| Err(CaptureError::Failed(error)))
}
