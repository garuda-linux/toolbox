//! Helpers shared by the unit tests.

use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use std::{env, fs, thread};

use tempfile::TempDir;

/// Whether `program` is found the way `Command` finds it, so a test of a
/// real adapter knows which answer to expect.
pub fn on_path(program: &str) -> bool {
    env::var_os("PATH")
        .is_some_and(|path| env::split_paths(&path).any(|folder| folder.join(program).is_file()))
}

/// Whether the tests run as root, who opens every file whatever its mode.
pub fn running_as_root() -> bool {
    rustix::process::geteuid().is_root()
}

/// Writes `contents` to `file` below `root`, with the folders above it.
pub fn write(root: &Path, file: &str, contents: impl AsRef<[u8]>) {
    let path = root.join(file);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// A shell script standing in for a program: prints `stdout` and `stderr`
/// and exits with `code`. The text must not hold a single quote.
pub fn fake_program(stdout: &str, stderr: &str, code: u8) -> (TempDir, PathBuf) {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("fake");

    runnable_script(
        &path,
        &format!("printf '%s' '{stdout}'\nprintf '%s' '{stderr}' >&2\nexit {code}"),
    );

    (folder, path)
}

/// Writes `body` as a shell script at `path` and returns once it can run.
/// A test thread that forks while the script is open for writing keeps a
/// copy of that descriptor until its child execs, and running the script
/// meanwhile fails with "Text file busy". The trial run ends before the
/// body, so it has no effect.
pub fn runnable_script(path: &Path, body: &str) {
    fs::write(
        path,
        format!("#!/bin/sh\n[ -n \"$TOOLBOX_WARM_UP\" ] && exit 0\n{body}\n"),
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();

    for _ in 0_u8..100_u8 {
        match Command::new(path).env("TOOLBOX_WARM_UP", "1").output() {
            Err(error) if error.kind() == ErrorKind::ExecutableFileBusy => {
                thread::sleep(Duration::from_millis(10));
            }
            Ok(_) | Err(_) => break,
        }
    }
}
