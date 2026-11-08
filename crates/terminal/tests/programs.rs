//! Real programs in a real pseudo-terminal feeding the terminal, with keys
//! and replies written back the way the Console does it.

use std::error::Error;
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use toolbox_pty::{Chunk, PtyCommand, Session};
use toolbox_terminal::{Key, Modifiers, Terminal, TerminalSize, encode_key};

const PATIENCE: Duration = Duration::from_secs(10);

const CONTROL: Modifiers = Modifiers {
    shift: false,
    alt: false,
    control: true,
};

const SIZE: TerminalSize = TerminalSize {
    columns: 80,
    rows: 24,
};

/// The programs below are on Arch and in the dev shell but not in the nix
/// build sandbox the flake's test check runs in.
const NEEDS_PROGRAMS: &str = "needs tput, less and micro on PATH (absent from the nix build sandbox), run with --include-ignored";

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    Reached,
    Closed,
    TimedOut,
}

fn start(command: PtyCommand) -> Result<(Session, Terminal), Box<dyn Error>> {
    Ok((
        Session::spawn(&command.size((SIZE.columns, SIZE.rows).into()))?,
        Terminal::new(SIZE),
    ))
}

fn drive<F>(session: &Session, terminal: &mut Terminal, until: F) -> Result<Outcome, Box<dyn Error>>
where
    F: Fn(&Terminal) -> bool,
{
    let deadline = Instant::now() + PATIENCE;

    while let Some(left) = deadline.checked_duration_since(Instant::now()) {
        if until(terminal) {
            return Ok(Outcome::Reached);
        }

        match session.output().recv_timeout(left) {
            Ok(Chunk::Text(text)) => {
                terminal.feed_str(&text);
                let replies = terminal.take_replies();
                if !replies.is_empty() {
                    session.input(&String::from_utf8(replies)?)?;
                }
            }
            Ok(Chunk::Closed) | Err(RecvTimeoutError::Disconnected) => {
                return Ok(if until(terminal) {
                    Outcome::Reached
                } else {
                    Outcome::Closed
                });
            }
            Err(RecvTimeoutError::Timeout) => break,
        }
    }

    Ok(Outcome::TimedOut)
}

fn type_key(session: &Session, terminal: &Terminal, key: Key, modifiers: Modifiers) -> TestResult {
    let bytes = encode_key(key, modifiers, terminal.key_mode());
    session.input(&String::from_utf8(bytes)?)?;
    Ok(())
}

fn screen_text(terminal: &Terminal) -> String {
    terminal
        .view()
        .rows
        .iter()
        .map(|row| row.iter().map(|run| run.text.as_str()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn first_row(terminal: &Terminal) -> String {
    screen_text(terminal)
        .lines()
        .next()
        .unwrap_or_default()
        .trim_end()
        .to_owned()
}

fn closed(session: &Session, terminal: &mut Terminal) -> Result<Outcome, Box<dyn Error>> {
    drive(session, terminal, |_terminal| false)
}

#[test]
fn a_program_switching_screens_with_raw_sequences() -> TestResult {
    let (session, mut terminal) = start(PtyCommand::new("sh").args([
        "-c",
        r"printf 'main'; printf '\033[?1049h\033[H\033[2Jalt'; sleep 0.3; printf '\033[?1049l'",
    ]))?;

    let on_alt = drive(&session, &mut terminal, |terminal| {
        first_row(terminal) == "alt"
    })?;

    assert_eq!(on_alt, Outcome::Reached);

    assert_eq!(closed(&session, &mut terminal)?, Outcome::Closed);
    assert_eq!(first_row(&terminal), "main");
    Ok(())
}

#[test]
#[ignore = "needs tput, less and micro on PATH (absent from the nix build sandbox), run with --include-ignored"]
fn tput_enters_and_leaves_the_alternate_screen() -> TestResult {
    let (session, mut terminal) = start(PtyCommand::new("sh").args([
        "-c",
        "printf main; tput smcup; tput clear; printf hi; sleep 0.3; tput rmcup",
    ]))?;

    let on_alt = drive(&session, &mut terminal, |terminal| {
        first_row(terminal) == "hi"
    })?;

    assert_eq!(on_alt, Outcome::Reached, "{NEEDS_PROGRAMS}");

    assert_eq!(closed(&session, &mut terminal)?, Outcome::Closed);
    assert_eq!(first_row(&terminal), "main");
    Ok(())
}

#[test]
#[ignore = "needs tput, less and micro on PATH (absent from the nix build sandbox), run with --include-ignored"]
fn less_pages_a_file_and_quits_on_q() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("diff.txt");
    std::fs::write(&file, "first line\nsecond line\n")?;

    let (session, mut terminal) = start(
        PtyCommand::new("less")
            .arg(&file)
            .env("LESSHISTFILE", "-")
            .env("LESS", ""),
    )?;

    let shown = drive(&session, &mut terminal, |terminal| {
        screen_text(terminal).contains("second line")
    })?;

    assert_eq!(shown, Outcome::Reached, "{}", screen_text(&terminal));

    // less paints a short file just above its prompt line at the bottom.
    let rows: Vec<String> = screen_text(&terminal)
        .lines()
        .map(|row| row.trim_end().to_owned())
        .collect();

    assert_eq!(rows.len(), usize::from(SIZE.rows));
    assert_eq!(
        rows.get(rows.len() - 3..rows.len() - 1),
        Some(&["first line".to_owned(), "second line".to_owned()][..])
    );
    assert!(rows.last().is_some_and(|row| row.ends_with("(END)")));

    type_key(&session, &terminal, Key::Char('q'), Modifiers::default())?;
    assert_eq!(closed(&session, &mut terminal)?, Outcome::Closed);
    assert!(!screen_text(&terminal).contains("second line"));
    Ok(())
}

/// The case the terminal exists for: paru opens micro to review a diff.
#[test]
#[ignore = "needs tput, less and micro on PATH (absent from the nix build sandbox), run with --include-ignored"]
fn micro_edits_saves_and_quits_through_encoded_keys() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("PKGBUILD.diff");
    std::fs::write(&file, "review me\n")?;
    std::fs::create_dir_all(dir.path().join("config"))?;

    let (session, mut terminal) = start(
        PtyCommand::new("micro")
            .arg("-config-dir")
            .arg(dir.path().join("config"))
            .arg(&file),
    )?;

    let opened = drive(&session, &mut terminal, |terminal| {
        screen_text(terminal).contains("review me")
    })?;

    assert_eq!(opened, Outcome::Reached, "{}", screen_text(&terminal));

    for character in "ok ".chars() {
        type_key(
            &session,
            &terminal,
            Key::Char(character),
            Modifiers::default(),
        )?;
    }

    let typed = drive(&session, &mut terminal, |terminal| {
        screen_text(terminal).contains("ok review me")
    })?;

    assert_eq!(typed, Outcome::Reached, "{}", screen_text(&terminal));

    type_key(&session, &terminal, Key::Char('s'), CONTROL)?;
    let saved = drive(&session, &mut terminal, |_terminal| {
        std::fs::read_to_string(&file).is_ok_and(|text| text.starts_with("ok review me"))
    })?;

    assert_eq!(saved, Outcome::Reached, "{}", screen_text(&terminal));

    type_key(&session, &terminal, Key::Char('q'), CONTROL)?;
    assert_eq!(closed(&session, &mut terminal)?, Outcome::Closed);
    assert!(!screen_text(&terminal).contains("review me"));
    Ok(())
}
