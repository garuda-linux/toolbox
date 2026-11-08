use proptest::prelude::*;

use super::{grub_value, quote, set_grub_value, unset_grub_value};

const ARCH: &str = include_str!("../fixtures/arch-default-grub");
const GARUDA: &str = include_str!("../fixtures/garuda-default-grub");

fn sourced(value: &str) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("grub");

    std::fs::write(
        &file,
        set_grub_value("", "GRUB_CMDLINE_LINUX_DEFAULT", value),
    )
    .unwrap();

    std::process::Command::new("/bin/sh")
        .args([
            "-c",
            &format!(
                ". {}; printf '%s' \"$GRUB_CMDLINE_LINUX_DEFAULT\"",
                file.display()
            ),
        ])
        .output()
        .unwrap()
}

fn changed<'text>(before: &'text str, after: &'text str) -> Vec<(&'text str, &'text str)> {
    assert_eq!(before.lines().count(), after.lines().count());

    before
        .lines()
        .zip(after.lines())
        .filter(|(old, new)| old != new)
        .collect()
}

#[test]
fn values_are_read_as_a_shell_reads_them() {
    assert_eq!(grub_value(ARCH, "GRUB_DEFAULT").as_deref(), Some("0"));
    assert_eq!(grub_value(ARCH, "GRUB_TIMEOUT").as_deref(), Some("5"));
    assert_eq!(
        grub_value(ARCH, "GRUB_CMDLINE_LINUX_DEFAULT").as_deref(),
        Some("loglevel=3 quiet")
    );
    assert_eq!(grub_value(ARCH, "GRUB_CMDLINE_LINUX").as_deref(), Some(""));
    assert_eq!(
        grub_value(GARUDA, "GRUB_DEFAULT").as_deref(),
        Some("Garuda Linux")
    );
    assert_eq!(
        grub_value(GARUDA, "GRUB_EARLY_INITRD_LINUX_STOCK").as_deref(),
        Some("")
    );
    assert_eq!(
        grub_value(GARUDA, "GRUB_MENU_PICTURE").as_deref(),
        Some("/usr/share/grub/themes/garuda-dr460nized/background.png")
    );
}

#[test]
fn commented_and_missing_variables_have_no_value() {
    assert_eq!(grub_value(ARCH, "GRUB_THEME"), None);
    assert_eq!(grub_value(ARCH, "GRUB_SAVEDEFAULT"), None);
    assert_eq!(grub_value(ARCH, "GRUB_NO_SUCH"), None);
    assert_eq!(grub_value(ARCH, "GRUB_CMDLINE"), None);
    assert_eq!(grub_value("", "GRUB_DEFAULT"), None);
}

#[test]
fn every_kind_of_quoting_is_understood() {
    for (line, value) in [
        ("X=plain", "plain"),
        ("X='single $quoted `text`'", "single $quoted `text`"),
        ("X=\"double quoted\"", "double quoted"),
        ("X=\"an \\\"embedded\\\" quote\"", "an \"embedded\" quote"),
        (
            "X=\"a \\$ sign and a \\\\ and a \\` tick\"",
            "a $ sign and a \\ and a ` tick",
        ),
        ("X=\"it's\"", "it's"),
        ("X='it'\\''s'", "it's"),
        ("X='a \"b\" c'", "a \"b\" c"),
        ("X=a\\ b", "a b"),
        ("X=mixed'single'\"double\"", "mixedsingledouble"),
        ("X=\"keeps \\n as it is\"", "keeps \\n as it is"),
        ("X=", ""),
        ("X=''", ""),
        ("X=5 # seconds", "5"),
        ("X=\"a # b\"  # a comment", "a # b"),
        ("  export   X=\"indented\"", "indented"),
        ("\tX=tabbed\r\n", "tabbed"),
    ] {
        assert_eq!(grub_value(line, "X").as_deref(), Some(value), "{line}");
    }
}

/// A value with an expansion needs a shell. A line with a second command
/// or an open quote is not read at all.
#[test]
fn what_only_a_shell_can_tell_has_no_value() {
    for line in [
        "X=\"$(lsb_release -i -s 2> /dev/null || echo Debian)\"",
        "X=$OTHER",
        "X=\"${OTHER} more\"",
        "X=`uname -r`",
        "X=a; rm -rf /",
        "X=a && true",
        "X=a b",
        "X=\"open",
        "X='open",
        "X=a\\",
        "XY=1",
        "# X=1",
        "echo X=1",
    ] {
        assert_eq!(grub_value(line, "X"), None, "{line}");
    }
}

#[test]
fn the_last_assignment_wins() {
    let config = "X=1\nY=a\nX=2\n";

    assert_eq!(grub_value(config, "X").as_deref(), Some("2"));
}

#[test]
fn setting_changes_only_the_value_of_its_line() {
    let after = set_grub_value(ARCH, "GRUB_TIMEOUT", "10");

    assert_eq!(
        changed(ARCH, &after),
        [("GRUB_TIMEOUT=5", "GRUB_TIMEOUT=10")]
    );

    let with_splash = set_grub_value(
        ARCH,
        "GRUB_CMDLINE_LINUX_DEFAULT",
        "loglevel=3 quiet splash",
    );

    assert_eq!(
        changed(ARCH, &with_splash),
        [(
            "GRUB_CMDLINE_LINUX_DEFAULT=\"loglevel=3 quiet\"",
            "GRUB_CMDLINE_LINUX_DEFAULT=\"loglevel=3 quiet splash\""
        )]
    );
    assert!(with_splash.ends_with("#GRUB_DISABLE_OS_PROBER=false\n"));
}

#[test]
fn a_bare_value_stays_bare_only_while_it_is_a_plain_word() {
    assert_eq!(set_grub_value("X=0\n", "X", "saved"), "X=saved\n");
    assert_eq!(
        set_grub_value("X=0\n", "X", "Garuda Linux"),
        "X=\"Garuda Linux\"\n"
    );
    assert_eq!(set_grub_value("X=0\n", "X", ""), "X=\"\"\n");
    assert_eq!(set_grub_value("X='a b'\n", "X", "5"), "X=\"5\"\n");
}

#[test]
fn a_commented_out_variable_is_switched_on_in_place() {
    let after = set_grub_value(ARCH, "GRUB_SAVEDEFAULT", "true");

    assert_eq!(
        changed(ARCH, &after),
        [("#GRUB_SAVEDEFAULT=true", "GRUB_SAVEDEFAULT=true")]
    );

    let themed = set_grub_value(ARCH, "GRUB_THEME", "/usr/share/grub/themes/x/theme.txt");

    assert_eq!(
        changed(ARCH, &themed),
        [(
            "#GRUB_THEME=\"/path/to/gfxtheme\"",
            "GRUB_THEME=\"/usr/share/grub/themes/x/theme.txt\""
        )]
    );
    assert_eq!(
        set_grub_value("  # X=1  # why\n", "X", "2"),
        "  X=2  # why\n"
    );
}

#[test]
fn an_active_line_is_preferred_to_a_commented_one() {
    let config = "#X=old\nX=1\n#X=other\n";

    assert_eq!(set_grub_value(config, "X", "2"), "#X=old\nX=2\n#X=other\n");
}

#[test]
fn of_a_variable_set_twice_the_last_line_changes() {
    let config = "X=1\n# between\nX=2 # last\nY=3\n";
    let after = set_grub_value(config, "X", "9");

    assert_eq!(after, "X=1\n# between\nX=9 # last\nY=3\n");
    assert_eq!(grub_value(&after, "X").as_deref(), Some("9"));
}

#[test]
fn a_new_variable_goes_to_the_end() {
    assert_eq!(
        set_grub_value("A=1\n", "GRUB_BACKGROUND", "/a b.png"),
        "A=1\nGRUB_BACKGROUND=\"/a b.png\"\n"
    );
    assert_eq!(set_grub_value("A=1", "B", "2"), "A=1\nB=2\n");
    assert_eq!(set_grub_value("", "B", "2"), "B=2\n");
}

#[test]
fn export_indent_comment_and_line_ending_are_kept() {
    assert_eq!(
        set_grub_value("\texport X=\"a\"\t# note\r\nY=1\r\n", "X", "b c"),
        "\texport X=\"b c\"\t# note\r\nY=1\r\n"
    );

    let after = set_grub_value(GARUDA, "GRUB_MENU_PICTURE", "/a.png");

    assert_eq!(
        changed(GARUDA, &after),
        [(
            "export GRUB_MENU_PICTURE=\"/usr/share/grub/themes/garuda-dr460nized/background.png\"",
            "export GRUB_MENU_PICTURE=\"/a.png\""
        )]
    );
}

#[test]
fn a_line_only_a_shell_understands_is_never_rewritten() {
    let config = "X=a; echo b\nX=\"open\nmore\"\n";
    let after = set_grub_value(config, "X", "1");

    assert_eq!(after, format!("{config}X=1\n"));
}

#[test]
fn a_computed_value_is_replaced_whole() {
    assert_eq!(
        set_grub_value("X=\"$(lsb_release -i -s)\" # name\n", "X", "Garuda"),
        "X=\"Garuda\" # name\n"
    );
}

#[test]
fn embedded_quotes_are_written_so_that_a_shell_reads_them_back() {
    let value = "say \"hi\" for $5 with a \\ and a `tick` and it's";
    let after = set_grub_value("X=1\n", "X", value);

    assert_eq!(
        after,
        "X=\"say \\\"hi\\\" for \\$5 with a \\\\ and a \\`tick\\` and it's\"\n"
    );
    assert_eq!(grub_value(&after, "X").as_deref(), Some(value));
    assert_eq!(quote("a\"b"), "\"a\\\"b\"");
}

#[test]
fn unsetting_comments_every_assignment_out() {
    let config = "X=1\n  export X=\"2\" # two\nY=3\n#X=4\n";

    assert_eq!(
        unset_grub_value(config, "X"),
        "#X=1\n  #export X=\"2\" # two\nY=3\n#X=4\n"
    );
    assert_eq!(grub_value(&unset_grub_value(config, "X"), "X"), None);
    assert_eq!(unset_grub_value(ARCH, "GRUB_THEME"), ARCH);
}

#[test]
fn setting_what_is_set_changes_nothing() {
    assert_eq!(set_grub_value(ARCH, "GRUB_TIMEOUT", "5"), ARCH);
    assert_eq!(
        set_grub_value(GARUDA, "GRUB_DISABLE_OS_PROBER", "false"),
        GARUDA
    );
}

proptest! {
    #[test]
    fn any_value_reads_back_and_leaves_the_rest_alone(value in "[ -~]{0,60}") {
        let after = set_grub_value(ARCH, "GRUB_CMDLINE_LINUX", &value);

        prop_assert_eq!(grub_value(&after, "GRUB_CMDLINE_LINUX"), Some(value));
        prop_assert_eq!(changed(ARCH, &after).len(), usize::from(after != ARCH));
        prop_assert_eq!(
            grub_value(&after, "GRUB_CMDLINE_LINUX_DEFAULT"),
            Some("loglevel=3 quiet".to_owned())
        );
    }

    #[test]
    fn a_new_line_reads_back_too(value in "[ -~]{0,60}") {
        let after = set_grub_value("A=1\n", "GRUB_NEW", &value);

        prop_assert_eq!(grub_value(&after, "GRUB_NEW"), Some(value));
        prop_assert!(after.starts_with("A=1\n"));
        prop_assert_eq!(after.lines().count(), 2);
    }

    /// The written file sourced by a real shell: it sees exactly the
    /// value, so quoting holds against expansions and substitutions.
    #[test]
    fn written_values_come_back_from_a_shell_untouched(value in "[ -~]{0,60}") {
        let seen = sourced(&value);

        prop_assert!(seen.status.success());
        prop_assert_eq!(String::from_utf8(seen.stdout).unwrap(), value);
    }
}

#[test]
fn a_hostile_value_runs_nothing_when_sourced() {
    let dir = tempfile::tempdir().unwrap();
    let canary = dir.path().join("pwned");
    let value = format!("x$(touch {})y`id`\"$HOME\"; reboot #", canary.display());
    let seen = sourced(&value);

    assert!(seen.status.success());
    assert_eq!(String::from_utf8(seen.stdout).unwrap(), value);
    assert!(!canary.exists(), "the sourced value ran a command");
}

#[test]
fn unrelated_lines_pass_through_a_change() {
    assert_eq!(set_grub_value("# hello\nX=0\n", "X", "1"), "# hello\nX=1\n");
}
