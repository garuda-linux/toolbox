use super::{InvalidShellPath, MAX_SHELL_PATH_BYTES, ShellPath};

#[test]
fn shells_from_etc_shells_are_accepted() {
    for path in [
        "/bin/sh",
        "/usr/bin/bash",
        "/usr/bin/zsh",
        "/usr/bin/fish",
        "/usr/bin/git-shell",
    ] {
        assert_eq!(ShellPath::new(path).unwrap().as_str(), path);
    }
}

#[test]
fn a_shell_path_must_be_absolute() {
    assert_eq!(
        ShellPath::new("bash").unwrap_err(),
        InvalidShellPath::NotAbsolute("bash".to_owned())
    );
    assert_eq!(ShellPath::new("").unwrap_err(), InvalidShellPath::Empty);
}

#[test]
fn a_shell_path_cannot_climb_or_hide_components() {
    for path in [
        "/usr/../bin/sh",
        "/usr/./bin/sh",
        "/usr//bin/sh",
        "/usr/bin/",
        "/",
        "/..",
    ] {
        assert!(
            matches!(
                ShellPath::new(path),
                Err(InvalidShellPath::InvalidComponent(_))
            ),
            "{path}"
        );
    }
}

#[test]
fn a_shell_path_has_no_separator_space_or_control_character() {
    assert_eq!(
        ShellPath::new("/bin/a:b").unwrap_err(),
        InvalidShellPath::InvalidCharacter(':')
    );
    assert_eq!(
        ShellPath::new("/bin/a b").unwrap_err(),
        InvalidShellPath::InvalidCharacter(' ')
    );
    assert_eq!(
        ShellPath::new("/bin/sh\n").unwrap_err(),
        InvalidShellPath::InvalidCharacter('\n')
    );
}

#[test]
fn a_shell_path_has_a_length_limit() {
    let long = format!("/{}", "a".repeat(MAX_SHELL_PATH_BYTES));

    assert_eq!(
        ShellPath::new(long).unwrap_err(),
        InvalidShellPath::TooLong {
            max: MAX_SHELL_PATH_BYTES
        }
    );
}

#[test]
fn a_shell_path_is_checked_while_decoding() {
    assert_eq!(
        serde_json::from_str::<ShellPath>("\"/usr/bin/zsh\"").unwrap(),
        ShellPath::new("/usr/bin/zsh").unwrap()
    );
    serde_json::from_str::<ShellPath>("\"/tmp/../bin/sh\"").unwrap_err();
}

#[test]
fn a_shell_path_parses_from_str_and_borrows_as_str() {
    let parsed: ShellPath = "/usr/bin/zsh".parse().unwrap();

    assert_eq!(parsed.as_ref(), "/usr/bin/zsh");
    assert_eq!(parsed.to_string(), "/usr/bin/zsh");
    "/usr/bin/zsh\0".parse::<ShellPath>().unwrap_err();
}

#[test]
fn a_shell_is_named_by_its_program() {
    let shells = ["/usr/bin/fish", "/bin/sh"].map(|path| ShellPath::new(path).unwrap());

    assert_eq!(ShellPath::names(&shells), ["fish", "sh"]);
}

#[test]
fn two_paths_of_one_program_keep_their_paths() {
    // /etc/shells lists bash twice on Arch, by its two paths.
    let shells =
        ["/bin/bash", "/usr/bin/bash", "/usr/bin/zsh"].map(|path| ShellPath::new(path).unwrap());

    assert_eq!(
        ShellPath::names(&shells),
        ["/bin/bash", "/usr/bin/bash", "zsh"]
    );
}
