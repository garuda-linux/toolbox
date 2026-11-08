use toolbox_plan::{ConfigFilePath, ConfigText};

use super::super::session::Problem;
use super::{Viewed, Viewing};

#[derive(Debug, PartialEq, Eq)]
struct Shown {
    phase: &'static str,
    path: String,
    target: String,
    text: String,
    problem: &'static str,
}

fn path(text: &str) -> ConfigFilePath {
    ConfigFilePath::new(text).unwrap()
}

fn shown(viewing: &Viewing) -> Shown {
    let read = viewing.viewed();

    Shown {
        phase: viewing.phase().code(),
        path: viewing.path().map(ToString::to_string).unwrap_or_default(),
        target: read.map(|read| read.path.to_string()).unwrap_or_default(),
        text: read
            .map(|read| read.content.as_str().to_owned())
            .unwrap_or_default(),
        problem: viewing.problem().code,
    }
}

fn text(path_text: &str, content: &str) -> Viewed {
    Viewed {
        path: path(path_text),
        content: ConfigText::new(content).unwrap(),
    }
}

#[test]
fn a_read_shows_the_text_with_where_it_was_read() {
    let mut viewing = Viewing::default();

    assert_eq!(shown(&viewing).phase, "idle");

    let number = viewing.start(path("/etc/passwd")).unwrap();
    let reading = shown(&viewing);

    assert_eq!(
        (reading.phase, reading.path.as_str(), reading.text.as_str()),
        ("reading", "/etc/passwd", "")
    );

    viewing.read(
        number,
        Ok(text("/etc/real/passwd", "root:x:0:0::/root:/bin/bash\n")),
    );

    let view = shown(&viewing);

    assert_eq!(
        (
            view.phase,
            view.path.as_str(),
            view.target.as_str(),
            view.text.as_str(),
            view.problem
        ),
        (
            "shown",
            "/etc/passwd",
            "/etc/real/passwd",
            "root:x:0:0::/root:/bin/bash\n",
            ""
        )
    );

    viewing.close();
    assert_eq!(shown(&viewing), shown(&Viewing::default()));
}

#[test]
fn a_refused_read_says_why() {
    let mut viewing = Viewing::default();
    let number = viewing.start(path("/etc/sudoers")).unwrap();

    viewing.read(number, Err(Problem::new("dismissed", "")));

    let view = shown(&viewing);

    assert_eq!(
        (view.phase, view.problem, view.text.as_str()),
        ("failed", "dismissed", "")
    );
}

#[test]
fn a_second_view_waits_for_the_first_read() {
    let mut viewing = Viewing::default();

    viewing.start(path("/etc/passwd")).unwrap();
    assert_eq!(viewing.start(path("/etc/group")), Err("busy"));
    assert_eq!(shown(&viewing).path, "/etc/passwd");
}

/// The dialog closed while the helper still read: its late answer must
/// not show in the next dialog, of another file.
#[test]
fn a_late_answer_lands_in_no_other_view() {
    let mut viewing = Viewing::default();
    let first = viewing.start(path("/etc/passwd")).unwrap();

    viewing.close();

    let second = viewing.start(path("/etc/group")).unwrap();

    viewing.read(
        first,
        Ok(text("/etc/passwd", "root:x:0:0::/root:/bin/bash\n")),
    );
    assert_eq!(shown(&viewing).phase, "reading");

    viewing.read(second, Ok(text("/etc/group", "wheel:x:998:\n")));
    assert_eq!(shown(&viewing).text, "wheel:x:998:\n");

    viewing.close();
    viewing.read(second, Ok(text("/etc/group", "wheel:x:998:\n")));
    assert_eq!(shown(&viewing).phase, "idle");
}
