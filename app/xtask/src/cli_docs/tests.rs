use super::{files, write_or_check};

#[test]
fn every_shell_and_the_man_page_get_a_file() {
    let names: Vec<String> = files().into_iter().map(|(name, _)| name).take(4).collect();

    assert_eq!(
        names,
        [
            "garuda-toolbox.bash",
            "garuda-toolbox.fish",
            "_garuda-toolbox",
            "garuda-toolbox.1"
        ]
    );
}

#[test]
fn each_hook_asks_the_binary_on_the_path_for_its_shell() {
    for (name, bytes) in files().into_iter().take(3) {
        let text = String::from_utf8_lossy(&bytes);
        let shell = match name.as_str() {
            "garuda-toolbox.bash" => "bash",
            "garuda-toolbox.fish" => "fish",
            _ => "zsh",
        };

        assert!(
            text.contains(&format!("COMPLETE=\"{shell}\""))
                || text.contains(&format!("COMPLETE={shell}")),
            "{name} sets no COMPLETE={shell}"
        );
        assert!(
            text.contains("garuda-toolbox --") || text.contains("\"garuda-toolbox\" --"),
            "{name} does not call the binary"
        );
    }
}

#[test]
fn the_zsh_hook_answers_its_first_tab_when_autoloaded() {
    let made = files();
    let zsh = made
        .iter()
        .find(|(name, _)| name == "_garuda-toolbox")
        .unwrap();

    let text = String::from_utf8_lossy(&zsh.1);

    assert!(text.starts_with("#compdef garuda-toolbox"));
    assert!(text.contains("_clap_dynamic_completer_garuda_toolbox \"$@\""));
}

#[test]
fn the_man_page_knows_every_command() {
    let made = files();
    let main = made
        .iter()
        .find(|(name, _)| name == "garuda-toolbox.1")
        .unwrap();

    let text = String::from_utf8_lossy(&main.1);

    for command in ["status", "diagnostics", "update", "services"] {
        assert!(text.contains(command), "the man page lacks {command}");
    }
}

#[test]
fn check_finds_a_stale_file_and_write_fixes_it() {
    let dir = tempfile::tempdir().unwrap();

    write_or_check(dir.path(), true).unwrap_err();
    write_or_check(dir.path(), false).unwrap();
    write_or_check(dir.path(), true).unwrap();
}

#[test]
fn every_page_the_man_page_refers_to_is_made() {
    let made = files();
    let names: Vec<String> = made.iter().map(|(name, _)| name.clone()).collect();
    let main = made
        .iter()
        .find(|(name, _)| name == "garuda-toolbox.1")
        .map(|(_, bytes)| String::from_utf8_lossy(bytes).into_owned())
        .unwrap();

    for command in ["status", "services", "users"] {
        let page = format!("garuda-toolbox-{command}.1");
        let reference = format!("garuda\\-toolbox\\-{command}(1)");
        assert!(main.contains(&reference), "{page} is not referred to");
        assert!(names.contains(&page), "{page} is referred to but not made");
    }
}

fn pages() -> Vec<(String, String)> {
    files()
        .into_iter()
        .filter(|(name, _)| name.ends_with(".1"))
        .map(|(name, bytes)| (name, String::from_utf8_lossy(&bytes).into_owned()))
        .collect()
}

#[test]
fn every_page_any_page_refers_to_is_made() {
    let pages = pages();
    let names: Vec<&str> = pages.iter().map(|(name, _)| name.as_str()).collect();

    for (name, text) in &pages {
        for word in text.split_whitespace() {
            let Some(page) = word
                .trim_end_matches(',')
                .strip_suffix("(1)")
                .filter(|page| page.starts_with("garuda\\-toolbox"))
            else {
                continue;
            };

            let file = format!("{}.1", page.replace("\\-", "-"));
            assert!(names.contains(&file.as_str()), "{name} refers to {file}");
        }
    }
}

#[test]
fn the_deepest_commands_get_a_page() {
    let pages = pages();
    let names: Vec<&str> = pages.iter().map(|(name, _)| name.as_str()).collect();

    for page in [
        "garuda-toolbox-services-list.1",
        "garuda-toolbox-network-drivers-blacklist.1",
        "garuda-toolbox-locales-packs-install.1",
    ] {
        assert!(names.contains(&page), "{page} is not made");
    }
}

#[test]
fn every_page_names_the_version_and_its_relatives() {
    let version = format!("\"garuda-toolbox {}\"", env!("CARGO_PKG_VERSION"));

    for (name, text) in pages() {
        let title = text.lines().find(|line| line.starts_with(".TH")).unwrap();

        assert!(title.contains(&version), "{name}: {title}");
        assert!(text.contains(".SH \"SEE ALSO\""), "{name} has no SEE ALSO");
    }
}

#[test]
fn a_page_of_a_command_refers_to_its_parent() {
    let pages = pages();
    let list = pages
        .iter()
        .find(|(name, _)| name == "garuda-toolbox-services-list.1")
        .unwrap();

    let see_also = list.1.split(".SH \"SEE ALSO\"").nth(1).unwrap();

    assert!(see_also.contains("garuda\\-toolbox\\-services(1)"));
    assert!(see_also.contains("garuda\\-toolbox(1)"));
}

#[test]
fn the_synopsis_shows_what_an_option_takes() {
    let pages = pages();
    let services = pages
        .iter()
        .find(|(name, _)| name == "garuda-toolbox-services.1")
        .unwrap();

    let synopsis = services
        .1
        .lines()
        .skip_while(|line| *line != ".SH SYNOPSIS")
        .nth(1)
        .unwrap();

    assert!(
        synopsis.contains("[\\fB\\-\\-color\\fR \\fIWHEN\\fR]"),
        "{synopsis}"
    );
    assert!(
        synopsis.contains("[\\fB\\-\\-width\\fR \\fICOLUMNS\\fR]"),
        "{synopsis}"
    );
    assert!(
        synopsis.contains("[\\fB\\-q\\fR|\\fB\\-\\-quiet\\fR]"),
        "{synopsis}"
    );
}

#[test]
fn the_man_page_lists_the_environment() {
    let pages = pages();
    let main = pages
        .iter()
        .find(|(name, _)| name == "garuda-toolbox.1")
        .unwrap();

    let environment = main.1.split(".SH ENVIRONMENT").nth(1).unwrap();
    for variable in [
        "GARUDA_TOOLBOX_PAGER",
        "NO_COLOR",
        "CLICOLOR",
        "TERM",
        "COMPLETE",
    ] {
        assert!(environment.contains(variable), "{variable} is missing");
    }

    for developer in ["GARUDA_TOOLBOX_CATALOG_DIR", "GARUDA_TOOLBOX_HELPER"] {
        assert!(!environment.contains(developer), "{developer} is listed");
    }
}

#[test]
fn the_hooks_are_what_the_completions_command_prints() {
    let made = files();

    for (shell, name) in [
        (clap_complete::Shell::Bash, "garuda-toolbox.bash"),
        (clap_complete::Shell::Fish, "garuda-toolbox.fish"),
        (clap_complete::Shell::Zsh, "_garuda-toolbox"),
    ] {
        let mut printed = Vec::new();
        toolbox_cli::complete::registration(shell, &mut printed);

        let hook = made.iter().find(|(file, _)| file == name).unwrap();
        assert_eq!(hook.1, printed, "{name}");
    }
}

#[test]
fn a_page_names_the_other_names_of_its_command() {
    let pages = pages();
    let page = |file: &str| {
        pages
            .iter()
            .find(|(name, _)| name == file)
            .map(|(_, text)| text.clone())
            .unwrap()
    };

    for (file, alias) in [
        (
            "garuda-toolbox-packages-search.1",
            "garuda\\-toolbox packages list",
        ),
        (
            "garuda-toolbox-network-drivers-allow.1",
            "garuda\\-toolbox network drivers unblock",
        ),
        (
            "garuda-toolbox-kernels-remove.1",
            "garuda\\-toolbox kernels uninstall",
        ),
        ("garuda-toolbox-dkms-list.1", "garuda\\-toolbox dkms status"),
    ] {
        let aliases = page(file);
        let aliases = aliases.split(".SH ALIASES").nth(1).unwrap_or_default();

        assert!(aliases.contains(alias), "{file} does not name {alias}");
    }

    assert!(!page("garuda-toolbox-services-list.1").contains(".SH ALIASES"));
}
