use super::HomeWrite;
use crate::action::testing::shown_user;
use crate::procedure::{Builtin, Invocation};
use crate::{ConfigText, Fingerprint, HomePath, UserAction};

fn write(path: &str, content: &str, original: &[u8]) -> HomeWrite {
    HomeWrite {
        path: HomePath::new(path).unwrap(),
        content: ConfigText::new(content).unwrap(),
        original: Fingerprint::of(original),
    }
}

#[test]
fn a_home_write_is_one_builtin_run_as_the_user() {
    let write = write(".config/fish/config.fish", "set -x A 1\n", b"old");
    let action = UserAction::WriteHomeFile {
        write: write.clone(),
    };

    assert_eq!(
        action.procedure(),
        [Invocation::Builtin(Builtin::WriteHomeFile(write))]
    );
}

#[test]
fn a_home_write_shows_its_file_size_and_backup() {
    let action = UserAction::WriteHomeFile {
        write: write(".bashrc", "a = 1\n", b"old"),
    };

    assert_eq!(
        shown_user(&action),
        [
            "write ~/.bashrc (6 bytes), the old one kept as ~/.local/state/garuda-toolbox/backups/.bashrc  # only when it did not change meanwhile"
        ]
    );
}

#[test]
fn the_backup_keeps_the_files_place_below_the_backup_folder() {
    assert_eq!(
        write(".config/kitty/kitty.conf", "", b"").backup(),
        "garuda-toolbox/backups/.config/kitty/kitty.conf"
    );
}
