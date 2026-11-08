use super::{EditCopy, Editor, InvalidEditCopy};
use crate::UserAction;
use crate::action::testing::shown_user;

const COPY: &str = "/run/user/1000/garuda-toolbox/edit/fstab";

fn copy(text: &str) -> EditCopy {
    EditCopy::new(text).unwrap()
}

#[test]
fn a_variable_picks_an_editor_by_its_program_name() {
    let picks = [
        ("micro", Editor::Micro),
        ("nano", Editor::Nano),
        ("vim", Editor::Vim),
        ("nvim", Editor::Neovim),
        ("vi", Editor::Vi),
        ("helix", Editor::Helix),
        ("hx", Editor::Helix),
        ("emacs", Editor::Emacs),
        ("ne", Editor::Ne),
        ("joe", Editor::Joe),
        ("mcedit", Editor::Mcedit),
        ("/usr/local/bin/vim", Editor::Vim),
        ("  nvim  ", Editor::Neovim),
        ("emacs -nw", Editor::Emacs),
        ("/opt/x/nano --rcfile /tmp/evil", Editor::Nano),
    ];

    for (value, editor) in picks {
        assert_eq!(Editor::from_variable(value), Some(editor), "{value}");
    }
}

#[test]
fn anything_else_in_a_variable_picks_nothing() {
    for value in [
        "",
        "   ",
        "code --wait",
        "kate",
        "gedit",
        "sh -c 'rm -rf ~'",
        "vim;rm",
        "nano&&id",
        "Vim",
        "sudoedit",
        "/usr/bin/",
        "env vim",
    ] {
        assert_eq!(Editor::from_variable(value), None, "{value}");
    }
}

#[test]
fn every_editor_has_a_name_and_a_fixed_path() {
    for editor in Editor::ALL {
        let path = editor.program().path();

        assert!(path.starts_with("/usr/bin/"), "{path}");
        assert_eq!(
            Editor::from_variable(editor.names().first().copied().unwrap()),
            Some(editor)
        );
    }

    assert_eq!(Editor::Neovim.program().path(), "/usr/bin/nvim");
}

#[test]
fn the_editor_opens_the_copy_and_emacs_stays_in_the_terminal() {
    let edit = |editor| {
        shown_user(&UserAction::EditFile {
            editor,
            copy: copy(COPY),
        })
    };

    assert_eq!(edit(Editor::Micro), [format!("micro {COPY}")]);
    assert_eq!(edit(Editor::Neovim), [format!("nvim {COPY}")]);
    assert_eq!(edit(Editor::Emacs), [format!("emacs -nw {COPY}")]);
}

#[test]
fn a_copy_is_a_file_directly_in_an_edit_folder() {
    for text in [
        COPY,
        "/home/user/.cache/garuda-toolbox/edit/grub",
        "/run/user/1000/garuda-toolbox/edit/20-gpu.conf",
        "/run/user/1000/garuda-toolbox/edit/a file",
    ] {
        assert_eq!(copy(text).as_str(), text);
    }

    for text in [
        "/etc/fstab",
        "/run/user/1000/garuda-toolbox/fstab",
        "/run/user/1000/garuda-toolbox/edit",
        "/run/user/1000/garuda-toolbox/edit/sub/fstab",
        "/run/user/1000/xgaruda-toolbox/edit/fstab",
        "/garuda-toolbox/edit",
    ] {
        assert_eq!(
            EditCopy::new(text).unwrap_err(),
            InvalidEditCopy::OutsideEditFolder(text.to_owned()),
            "{text}"
        );
    }
}

#[test]
fn a_copy_cannot_climb_or_be_relative() {
    assert_eq!(
        EditCopy::new("run/garuda-toolbox/edit/x").unwrap_err(),
        InvalidEditCopy::Relative("run/garuda-toolbox/edit/x".to_owned())
    );
    assert_eq!(
        EditCopy::new("-rf").unwrap_err(),
        InvalidEditCopy::Relative("-rf".to_owned())
    );

    for text in [
        "/run/garuda-toolbox/edit/../../../etc/fstab",
        "/run/garuda-toolbox/edit/..",
        "/run/garuda-toolbox/edit/",
        "/run//garuda-toolbox/edit/x",
        "/run/./garuda-toolbox/edit/x",
    ] {
        assert_eq!(
            EditCopy::new(text).unwrap_err(),
            InvalidEditCopy::InvalidComponent(text.to_owned()),
            "{text}"
        );
    }

    assert_eq!(
        EditCopy::new("/run/garuda-toolbox/edit/a\nb").unwrap_err(),
        InvalidEditCopy::InvalidCharacter('\n')
    );
    assert_eq!(
        EditCopy::new(format!(
            "/{}/garuda-toolbox/edit/x",
            "a".repeat(super::MAX_PATH_BYTES)
        ))
        .unwrap_err(),
        InvalidEditCopy::TooLong {
            max: super::MAX_PATH_BYTES
        }
    );
}

#[test]
fn an_edit_action_is_checked_again_when_it_is_decoded() {
    let action = UserAction::EditFile {
        editor: Editor::Helix,
        copy: copy(COPY),
    };

    let json = serde_json::to_string(&action).unwrap();

    assert_eq!(
        json,
        format!("{{\"kind\":\"edit-file\",\"editor\":\"helix\",\"copy\":\"{COPY}\"}}")
    );
    assert_eq!(serde_json::from_str::<UserAction>(&json).unwrap(), action);
    serde_json::from_str::<UserAction>(&json.replace("helix", "bash")).unwrap_err();
    serde_json::from_str::<UserAction>(&json.replace(COPY, "/etc/fstab")).unwrap_err();
}

#[test]
fn a_copy_parses_from_str_and_borrows_as_str() {
    let parsed: EditCopy = COPY.parse().unwrap();

    assert_eq!(parsed.as_ref(), COPY);
    assert_eq!(parsed.to_string(), COPY);
    assert_eq!(format!("{parsed:?}"), format!("{COPY:?}"));
    format!("{COPY}\0").parse::<EditCopy>().unwrap_err();
}
