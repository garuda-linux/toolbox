//! Starting a program with a new pseudo-terminal as its controlling
//! terminal, with the failing stage named in every error.

use std::fs::File;
use std::os::fd::OwnedFd;
use std::os::unix::process::CommandExt as _;
use std::path::Path;
use std::process::{Child, Stdio};
use std::{io, ptr};

use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};

use crate::PtyCommand;
use crate::size::winsize;

/// A new pseudo-terminal master, closed in children.
///
/// # Errors
///
/// When the system has no free pseudo-terminal or `/dev/ptmx` is missing.
pub fn open_master() -> io::Result<OwnedFd> {
    Ok(openpt(
        OpenptFlags::RDWR | OpenptFlags::NOCTTY | OpenptFlags::CLOEXEC,
    )?)
}

/// `error` with the `stage` that failed in front. A bare errno never says
/// which step went wrong.
pub fn stage_error<E>(stage: &str, error: E) -> io::Error
where
    E: Into<io::Error>,
{
    let error = error.into();
    io::Error::new(error.kind(), format!("{stage}: {error}"))
}

/// Starts `command` with the slave side as stdin, stdout and stderr and as
/// its controlling terminal. Returns the child and the master side.
///
/// # Errors
///
/// When a terminal stage or the spawn itself fails. The message names the
/// stage.
pub fn spawn(command: &PtyCommand) -> io::Result<(Child, File)> {
    let master = open_master().map_err(|error| stage_error("open a terminal master", error))?;
    grantpt(&master).map_err(|error| stage_error("grant a terminal", error))?;
    unlockpt(&master).map_err(|error| stage_error("unlock a terminal", error))?;

    // Input to a program that stopped reading would otherwise block the
    // thread that also watches for a cancel.
    rustix::io::ioctl_fionbio(&master, true)
        .map_err(|error| stage_error("make a terminal non-blocking", error))?;
    let name =
        ptsname(&master, Vec::new()).map_err(|error| stage_error("name a terminal", error))?;

    let slave = File::options()
        .read(true)
        .write(true)
        .open(Path::new(&*name.to_string_lossy()))
        .map_err(|error| stage_error("open a terminal slave", error))?;

    rustix::termios::tcsetwinsize(&slave, winsize(command.terminal_size()))
        .map_err(|error| stage_error("size a terminal", error))?;

    let mut process = command.to_command();
    process
        .stdin(Stdio::from(slave.try_clone()?))
        .stdout(Stdio::from(slave.try_clone()?))
        .stderr(Stdio::from(slave));

    #[expect(
        unsafe_code,
        reason = "pre_exec runs in the child between fork and exec"
    )]
    // SAFETY: only async-signal-safe syscalls run between fork and exec:
    // the default SIGINT and SIGQUIT and no blocked signals, a new
    // session, then the terminal on stdin as its controlling tty.
    unsafe {
        process.pre_exec(|| {
            default_signals()?;
            rustix::process::setsid()?;
            rustix::process::ioctl_tiocsctty(rustix::stdio::stdin())?;
            Ok(())
        });
    }

    let child = process
        .spawn()
        .map_err(|error| stage_error(&format!("start {}", command.program().display()), error))?;

    Ok((child, File::from(master)))
}

/// An app started with SIGINT ignored, as a script's `&` does, passes that
/// on to every program it starts, which then shrugs off Cancel's Ctrl+C.
/// The program gets the defaults a terminal gives it.
#[expect(
    unsafe_code,
    reason = "signal dispositions and the mask have no safe wrapper in rustix"
)]
fn default_signals() -> io::Result<()> {
    for signal in [libc::SIGINT, libc::SIGQUIT] {
        // SAFETY: `signal` is async-signal-safe and touches no memory.
        let previous = unsafe { libc::signal(signal, libc::SIG_DFL) };
        if previous == libc::SIG_ERR {
            return Err(io::Error::last_os_error());
        }
    }

    // SAFETY: an all-zero `sigset_t` is a valid value to fill.
    let mut none = unsafe { std::mem::zeroed::<libc::sigset_t>() };

    // SAFETY: `none` lives on this frame for the call.
    unsafe { libc::sigemptyset(&raw mut none) };

    // SAFETY: async-signal-safe, `none` lives on this frame for the call.
    let set = unsafe { libc::sigprocmask(libc::SIG_SETMASK, &raw const none, ptr::null_mut()) };
    if set != 0_i32 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
