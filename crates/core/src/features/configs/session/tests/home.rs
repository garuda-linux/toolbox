use toolbox_plan::{ConfigText, EditCopy, Editor, HomePath, HomeWrite};

use super::changed;
use crate::features::configs::session::{EditWrite, EditedFile, Fetched, Session, Work};

const OLD: &str = "set a 1\n";
const NEW: &str = "set a 2\n";

fn home() -> HomePath {
    HomePath::new(".config/fish/config.fish").unwrap()
}

fn text(content: &str) -> ConfigText {
    ConfigText::new(content).unwrap()
}

fn reviewing() -> Session {
    let mut session = Session::default();
    session.start(home()).unwrap();
    session.fetched(Ok(Fetched {
        editor: Editor::Nano,
        copy: EditCopy::new("/run/user/1000/garuda-toolbox/edit/config.fish").unwrap(),
        original: text(OLD),
        target: home().into(),
    }));
    session.plan_started();
    session.plan_finished(true);
    session.compared(Ok(changed(OLD, NEW)));
    session
}

#[test]
fn a_file_of_the_user_is_read_as_its_home_path() {
    let mut session = Session::default();

    assert_eq!(
        session.start(home()),
        Ok(Work::Fetch(EditedFile::Home(home())))
    );
    assert_eq!(
        session.path().map(ToString::to_string).as_deref(),
        Some("~/.config/fish/config.fish")
    );
}

#[test]
fn a_file_of_the_user_keeps_its_backup_in_the_state_folder() {
    assert_eq!(
        reviewing().backup().as_deref(),
        Some("~/.local/state/garuda-toolbox/backups/.config/fish/config.fish")
    );
}

#[test]
fn saving_a_file_of_the_user_writes_it_as_the_user() {
    assert_eq!(
        reviewing().apply(),
        Some(Work::Write(EditWrite::Home(HomeWrite {
            path: home(),
            content: text(NEW),
            original: text(OLD).fingerprint(),
        })))
    );
}
