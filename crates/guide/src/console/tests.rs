use super::{ConsoleLine, console_lines};

mod words {
    use super::super::{Word, command_words};

    fn words(command: &str) -> Vec<(Word, &str)> {
        let parts = command_words(command);
        let joined: String = parts.iter().map(|(_, text)| *text).collect();
        assert_eq!(joined, command, "the words add up to the command");
        parts
    }

    fn marked(command: &str) -> Vec<(Word, &str)> {
        words(command)
            .into_iter()
            .filter(|(word, _)| *word != Word::Plain)
            .collect()
    }

    /// `boot-options.md`.
    #[test]
    fn a_kernel_parameter_shows_its_value_apart_from_its_key() {
        let command = "garuda-toolbox boot param add systemd.unit=rescue.target --force\n";

        assert_eq!(
            words(command),
            [
                (Word::Program, "garuda-toolbox"),
                (Word::Plain, " "),
                (Word::Plain, "boot"),
                (Word::Plain, " "),
                (Word::Plain, "param"),
                (Word::Plain, " "),
                (Word::Plain, "add"),
                (Word::Plain, " "),
                (Word::Key, "systemd.unit="),
                (Word::Value, "rescue.target"),
                (Word::Plain, " "),
                (Word::Flag, "--force"),
                (Word::Plain, "\n"),
            ]
        );
    }

    /// `boot-repair.md`: a device stays plain.
    #[test]
    fn flags_after_a_device_are_flags() {
        let command = "garuda-toolbox bootrepair regenerate /dev/sdb2 --boot-repair --trust-disk";

        assert_eq!(
            marked(command),
            [
                (Word::Program, "garuda-toolbox"),
                (Word::Flag, "--boot-repair"),
                (Word::Flag, "--trust-disk"),
            ]
        );
    }

    /// `cli.md`: the value after a flag is a plain word.
    #[test]
    fn a_flag_value_after_a_space_stays_plain() {
        let command = "garuda-toolbox users edit ada --shell /usr/bin/fish --add-group docker";

        assert_eq!(
            marked(command),
            [
                (Word::Program, "garuda-toolbox"),
                (Word::Flag, "--shell"),
                (Word::Flag, "--add-group"),
            ]
        );
    }

    /// `cli.md`: a quoted name keeps its space and stays plain.
    #[test]
    fn a_quoted_name_stays_plain() {
        let command = "garuda-toolbox users add grace --full-name \"Grace Hopper\" --admin";
        let parts = words(command);

        assert!(parts.contains(&(Word::Plain, "\"Grace")), "{parts:?}");
        assert!(parts.contains(&(Word::Plain, "Hopper\"")), "{parts:?}");
        assert_eq!(
            marked(command),
            [
                (Word::Program, "garuda-toolbox"),
                (Word::Flag, "--full-name"),
                (Word::Flag, "--admin"),
            ]
        );
    }

    /// `cli.md`: an assignment before the program is not the program.
    #[test]
    fn an_environment_assignment_comes_before_the_program() {
        let command = "EDITOR=nano garuda-toolbox configs edit pacman-config";

        assert_eq!(
            marked(command),
            [
                (Word::Key, "EDITOR="),
                (Word::Value, "nano"),
                (Word::Program, "garuda-toolbox"),
            ]
        );
    }

    /// `cli.md`: the redirect and its file stay plain.
    #[test]
    fn a_redirect_stays_plain() {
        let command = "garuda-toolbox completions zsh > ~/.zfunc/_garuda-toolbox";

        assert_eq!(marked(command), [(Word::Program, "garuda-toolbox")]);
    }

    /// `cli.md`: a program alone, without a line break.
    #[test]
    fn a_program_alone_is_one_word() {
        assert_eq!(words("garuda-toolbox"), [(Word::Program, "garuda-toolbox")]);
    }

    #[test]
    fn an_empty_command_has_no_words() {
        assert_eq!(words(""), []);
    }
}

#[test]
fn a_prompt_line_parts_its_prompt_from_the_command() {
    let lines: Vec<_> = console_lines("$ garuda-toolbox status\n").collect();

    assert_eq!(
        lines,
        [ConsoleLine::Command {
            prompt: "$ ",
            command: "garuda-toolbox status\n",
        }]
    );
}

#[test]
fn a_line_without_the_prompt_is_output() {
    let lines: Vec<_> = console_lines("$ ls\nREADME.md\n").collect();

    assert_eq!(lines[1], ConsoleLine::Output("README.md\n"));
}

#[test]
fn a_root_prompt_or_a_dollar_without_its_space_is_output() {
    let text = "# cp a b\n$HOME\n";
    let lines: Vec<_> = console_lines(text).collect();

    assert_eq!(
        lines,
        [
            ConsoleLine::Output("# cp a b\n"),
            ConsoleLine::Output("$HOME\n"),
        ]
    );
}

#[test]
fn an_empty_line_is_output() {
    let lines: Vec<_> = console_lines("$ a\n\n$ b").collect();

    assert_eq!(lines[1], ConsoleLine::Output("\n"));
    assert_eq!(
        lines[2],
        ConsoleLine::Command {
            prompt: "$ ",
            command: "b",
        }
    );
}

#[test]
fn a_dollar_inside_a_command_stays_in_the_command() {
    let lines: Vec<_> = console_lines("$ echo $HOME $ PATH\n").collect();

    assert_eq!(
        lines,
        [ConsoleLine::Command {
            prompt: "$ ",
            command: "echo $HOME $ PATH\n",
        }]
    );
}

/// Selection and copy take the block's text, so the parts must add up
/// to it.
#[test]
fn the_parts_add_up_to_the_text() {
    let text = "$ EDITOR=nano garuda-toolbox configs edit pacman-config\nsaved\n\n# done";
    let joined: String = console_lines(text)
        .flat_map(|line| match line {
            ConsoleLine::Command { prompt, command } => [prompt, command],
            ConsoleLine::Output(output) => [output, ""],
        })
        .collect();

    assert_eq!(joined, text);
}
