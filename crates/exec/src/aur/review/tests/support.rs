//! Fixtures of the review tests: a home, a source repository per base and a
//! stand-in for paru that copies that repository into the clone.

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};

use tempfile::TempDir;
use toolbox_plan::protocol::{Answer, Event, Outcome};
use toolbox_plan::{AUR_CLONE, PackageName, StepId, TargetList};

use crate::Environment;
use crate::apply::EventSink;
use crate::aur::review::run;
use crate::pty_runner::PtyRunner;
use crate::test_support::{ada_home, printed};

pub const OCTOPI: &str = "octopi-git";
pub const PKGBUILD: &str = "pkgname=octopi-git\npkgver=1\n";
pub const QUESTION: &str = "Type y and press Enter";
const REVIEWED: &str = "refs/garuda-toolbox/reviewed";

#[derive(Clone, Copy)]
pub enum Fetch {
    /// What `paru -G` does: a clone the first time. After that only a fetch,
    /// which leaves `HEAD` and the files where they were.
    Copy,
    /// A clone whose own git config something local rewrote, such as an
    /// earlier build: the source's config copied over it.
    Tampered,
    /// Nothing at all: the base folder is missing.
    Nothing,
    /// A folder without the repository's files.
    Folder,
    /// A folder with the PKGBUILD only, no repository.
    PkgbuildOnly,
    /// A link to the repository.
    Link,
    /// paru fails.
    Fail,
    /// Prints an ESC sequence and U+202E in a line, then copies as `Copy`.
    Noisy,
    /// Prints more than the fetch cap.
    Flood,
    /// Runs until it is stopped.
    Sleep,
}

/// What a trigger does when the Console shows its text.
pub enum Then {
    Send(Answer),
    Stop(Arc<AtomicBool>),
    Drop,
    Act(Box<dyn FnOnce()>),
}

/// The Console: keeps what was shown and fires each trigger once.
pub struct Session {
    pub events: Vec<Event>,
    pub app: Option<Sender<Answer>>,
    pub triggers: Vec<(&'static str, Then)>,
}

impl Session {
    fn fire(&mut self, then: Then) {
        match then {
            Then::Send(answer) => {
                if let Some(app) = &self.app {
                    app.send(answer).unwrap();
                }
            }
            Then::Stop(flag) => flag.store(true, Ordering::Relaxed),
            Then::Drop => self.app = None,
            Then::Act(act) => act(),
        }
    }
}

impl EventSink for Session {
    fn send(&mut self, event: Event) {
        if let Event::Output { step: _, data } = &event {
            let mut index = 0;

            while index < self.triggers.len() {
                if data.contains(self.triggers[index].0) {
                    let (_, then) = self.triggers.remove(index);
                    self.fire(then);
                } else {
                    index += 1;
                }
            }
        }

        self.events.push(event);
    }
}

pub struct Case {
    pub root: TempDir,
    pub home: PathBuf,
    pub sources: PathBuf,
    pub log: PathBuf,
    pub paru: PathBuf,
    pub git: PathBuf,
}

impl Case {
    pub fn clone_of(&self, base: &str) -> PathBuf {
        self.home.join(AUR_CLONE).join(base)
    }
}

pub fn enter() -> Answer {
    Answer::Input {
        data: "\r".to_owned(),
    }
}

pub fn yes() -> Answer {
    Answer::Input {
        data: "y\r".to_owned(),
    }
}

pub fn enter_at(text: &'static str) -> (&'static str, Then) {
    (text, Then::Send(enter()))
}

pub fn yes_at(text: &'static str) -> (&'static str, Then) {
    (text, Then::Send(yes()))
}

pub fn git_program() -> PathBuf {
    std::env::var_os("PATH")
        .and_then(|path| {
            std::env::split_paths(&path)
                .map(|folder| folder.join("git"))
                .find(|candidate| candidate.is_file())
        })
        .expect("git is on PATH")
}

/// Runs git in `folder` with a fixed identity, and returns its output.
pub fn git(folder: &Path, args: &[&str]) -> String {
    let output = Command::new(git_program())
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .current_dir(folder)
        .args([
            "-c",
            "user.name=ada",
            "-c",
            "user.email=ada@example.invalid",
            "-c",
            "commit.gpgSign=false",
        ])
        .args(args)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

/// Writes `files`, commits them all and returns the new commit id.
pub fn commit(folder: &Path, files: &[(&str, &str)]) -> String {
    for (name, content) in files {
        fs::write(folder.join(name), content).unwrap();
    }

    git(folder, &["add", "--all"]);
    git(folder, &["commit", "-q", "--allow-empty", "-m", "change"]);
    git(folder, &["rev-parse", "HEAD"])
}

fn repository(folder: &Path, files: &[(&str, &str)]) {
    fs::create_dir_all(folder).unwrap();
    git(folder, &["init", "-q"]);
    commit(folder, files);
}

/// An executable that touches a marker file when git runs it as a program.
/// Returns the program and the marker.
pub fn marker_program(dir: &Path, name: &str) -> (PathBuf, PathBuf) {
    let marker = dir.join(format!("{name}.ran"));
    let program = dir.join(name);

    fs::write(&program, format!("#!/bin/sh\n: > '{}'\n", marker.display())).unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();

    (program, marker)
}

fn stand_in_paru(dir: &Path, log: &Path, sources: &Path, how: Fetch) -> PathBuf {
    let git = git_program();
    let fetched = format!(
        "if [ -d \"$base\"/.git ]; then '{git}' -C \"$base\" fetch -q; else '{git}' clone -q '{}'/\"$base\" \"$base\"; fi",
        sources.display(),
        git = git.display()
    );

    let step = match how {
        Fetch::Copy => fetched,
        Fetch::Tampered => format!(
            "{fetched} && cp '{}'/\"$base\"/.git/config \"$base\"/.git/config",
            sources.display()
        ),
        Fetch::Nothing => ":".to_owned(),
        Fetch::Folder => "mkdir -p \"$base\"".to_owned(),
        Fetch::PkgbuildOnly => format!(
            "mkdir -p \"$base\" && cp '{}'/\"$base\"/PKGBUILD \"$base\"/",
            sources.display()
        ),
        Fetch::Link => format!("ln -s '{}'/\"$base\" \"$base\"", sources.display()),
        Fetch::Fail => "exit 1".to_owned(),
        Fetch::Noisy => format!(
            "printf 'downloading: %s\\033[2J\\342\\200\\256 evil\\n' \"$base\" && {fetched}"
        ),
        Fetch::Flood => "head -c 131072 /dev/zero | tr '\\0' a".to_owned(),
        Fetch::Sleep => "sleep 30".to_owned(),
    };

    let program = dir.join("paru");
    let script = format!(
        "#!/bin/sh\nexport GIT_CONFIG_GLOBAL=/dev/null\nprintf '%s\\n' \"$(pwd -P)\" \"$@\" >> '{}'\nshift 2\nfor base in \"$@\"; do\n{step}\ndone\n",
        log.display()
    );

    fs::write(&program, script).unwrap();
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();

    program
}

pub fn case_with(how: Fetch, pkgbuild: &str) -> Case {
    case_with_bases(how, &[(OCTOPI, pkgbuild)])
}

/// One source repository per `(base, PKGBUILD)`, all fetched by one paru.
pub fn case_with_bases(how: Fetch, bases: &[(&str, &str)]) -> Case {
    let (root, home) = ada_home();
    fs::create_dir_all(home.join(".cache")).unwrap();
    let sources = root.path().join("sources");
    let log = root.path().join("paru.log");

    for (name, pkgbuild) in bases {
        repository(&sources.join(name), &[("PKGBUILD", pkgbuild)]);
    }

    let paru = stand_in_paru(root.path(), &log, &sources, how);

    Case {
        home,
        sources,
        log,
        paru,
        git: git_program(),
        root,
    }
}

pub fn case(how: Fetch) -> Case {
    case_with(how, PKGBUILD)
}

/// A review that reads its answers from `channel`, so a test can answer from
/// a thread of its own.
pub fn review_on(
    case: &Case,
    bases: &[&str],
    triggers: Vec<(&'static str, Then)>,
    stop: Option<Arc<AtomicBool>>,
    channel: (Sender<Answer>, Receiver<Answer>),
) -> (Outcome, Session) {
    let (tx, rx) = channel;
    let mut session = Session {
        events: Vec::new(),
        app: Some(tx),
        triggers,
    };

    let mut runner = PtyRunner::new(rx, Environment::Inherited).with_home(case.home.clone());

    if let Some(stop) = stop {
        runner = runner.stop_on(stop);
    }

    let targets =
        TargetList::new(bases.iter().map(|base| PackageName::new(*base).unwrap())).unwrap();

    let outcome = run(
        &mut runner,
        (&case.paru, &case.git),
        StepId::new(1),
        &targets,
        &mut session,
    );

    (outcome, session)
}

/// Runs a review of `bases` with `triggers`. The runner's stop flag is
/// `stop` when given.
pub fn review(
    case: &Case,
    bases: &[&str],
    triggers: Vec<(&'static str, Then)>,
    stop: Option<Arc<AtomicBool>>,
) -> (Outcome, Session) {
    review_on(case, bases, triggers, stop, channel())
}

/// What the Console showed, as plain text.
pub fn shown(session: &Session) -> String {
    printed(&session.events)
}

/// Whether the Console asked the question.
pub fn asked(session: &Session) -> bool {
    shown(session).contains(QUESTION)
}

/// The commit the app's ref of `base` names, if it has one.
pub fn reviewed(case: &Case, base: &str) -> Option<String> {
    let output = Command::new(git_program())
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .current_dir(case.clone_of(base))
        .args(["rev-parse", "--verify", "--quiet", REVIEWED])
        .output()
        .unwrap();

    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).unwrap().trim().to_owned())
}

pub fn head(case: &Case, base: &str) -> String {
    git(&case.clone_of(base), &["rev-parse", "HEAD"])
}
