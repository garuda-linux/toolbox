use std::ffi::OsString;

use super::{ColorEnv, ColorMode, Mark, Paint, Row, Style, style};

const PLAIN: Paint = Paint::new(Style::Plain);
const COLOR: Paint = Paint::new(Style::Color);

fn env(set: &[(&str, &str)]) -> ColorEnv {
    ColorEnv::read(|key| {
        set.iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| OsString::from(value))
    })
}

#[test]
fn colour_only_on_a_terminal_without_no_color() {
    assert_eq!(style(ColorMode::Auto, env(&[]), true), Style::Color);
    assert_eq!(style(ColorMode::Auto, env(&[]), false), Style::Plain);
    assert_eq!(
        style(ColorMode::Auto, env(&[("NO_COLOR", "1")]), true),
        Style::Plain
    );
}

#[test]
fn an_empty_no_color_does_not_count() {
    assert_eq!(
        style(ColorMode::Auto, env(&[("NO_COLOR", "")]), true),
        Style::Color
    );
}

#[test]
fn the_flag_wins_over_the_terminal_and_no_color() {
    assert_eq!(
        style(ColorMode::Always, env(&[("NO_COLOR", "1")]), false),
        Style::Color
    );
    assert_eq!(style(ColorMode::Never, env(&[]), true), Style::Plain);
}

#[test]
fn a_force_variable_colours_a_pipe() {
    for force in ["CLICOLOR_FORCE", "FORCE_COLOR"] {
        assert_eq!(
            style(ColorMode::Auto, env(&[(force, "1")]), false),
            Style::Color
        );
        assert_eq!(
            style(
                ColorMode::Auto,
                env(&[(force, "1"), ("NO_COLOR", "1")]),
                false
            ),
            Style::Color
        );
        assert_eq!(
            style(ColorMode::Auto, env(&[(force, "0")]), false),
            Style::Plain
        );
    }
}

#[test]
fn a_dumb_terminal_and_clicolor_0_get_no_colour() {
    assert_eq!(
        style(ColorMode::Auto, env(&[("TERM", "dumb")]), true),
        Style::Plain
    );
    assert_eq!(
        style(ColorMode::Auto, env(&[("CLICOLOR", "0")]), true),
        Style::Plain
    );
    assert_eq!(
        style(ColorMode::Auto, env(&[("CLICOLOR", "1")]), true),
        Style::Color
    );
    assert_eq!(
        style(ColorMode::Always, env(&[("TERM", "dumb")]), true),
        Style::Color
    );
}

#[test]
fn clicolor_force_wins_over_clicolor_0() {
    assert_eq!(
        style(
            ColorMode::Auto,
            env(&[("CLICOLOR", "0"), ("CLICOLOR_FORCE", "1")]),
            true
        ),
        Style::Color
    );
}

#[test]
fn a_title_carries_its_summary_in_brackets() {
    assert_eq!(PLAIN.title("Services", None), "Services");
    assert_eq!(
        PLAIN.title("Services", Some("142, 61 running")),
        "Services  (142, 61 running)"
    );
    assert_eq!(
        COLOR.title("Services", Some("2")),
        "\u{1b}[1mServices\u{1b}[0m  \u{1b}[2m(2)\u{1b}[0m"
    );
}

#[test]
fn an_item_is_indented_by_two() {
    assert_eq!(
        PLAIN.item("systemctl restart -- sshd.service"),
        "  systemctl restart -- sshd.service"
    );
}

#[test]
fn errors_warnings_and_hints_say_so_first() {
    assert_eq!(PLAIN.error("no such file"), "error: no such file");
    assert_eq!(PLAIN.warning("slow mirror"), "warning: slow mirror");
    assert_eq!(PLAIN.hint("run it again"), "hint: run it again");
    assert_eq!(
        COLOR.error("no such file"),
        "\u{1b}[1m\u{1b}[31merror:\u{1b}[0m no such file"
    );
    assert_eq!(
        COLOR.hint("run it again"),
        "\u{1b}[1m\u{1b}[36mhint:\u{1b}[0m run it again"
    );
}

#[test]
fn a_detail_is_indented_under_its_line() {
    assert_eq!(PLAIN.detail("one\ntwo\n"), "    one\n    two\n");
}

#[test]
fn a_plain_list_has_no_marks() {
    let text = PLAIN.list(
        "Services",
        Some("2, 1 running"),
        &[],
        &[
            Row::cells(Mark::On, &["sshd.service", "running", "at boot"]),
            Row::cells(Mark::Off, &["bluetooth.service", "stopped", ""]),
        ],
    );

    assert_eq!(
        text,
        "Services  (2, 1 running)\n\n  sshd.service       running  at boot\n  bluetooth.service  stopped\n"
    );
}

#[test]
fn a_coloured_list_marks_each_row_and_dims_the_later_columns() {
    let text = COLOR.list(
        "S",
        None,
        &[],
        &[
            Row::cells(Mark::On, &["a", "b"]),
            Row::cells(Mark::Off, &["cc", "d"]),
        ],
    );

    assert_eq!(
        text,
        "\u{1b}[1mS\u{1b}[0m\n\n  \u{1b}[32m●\u{1b}[0m a   \u{1b}[2mb\u{1b}[0m\n  \u{1b}[2m○\u{1b}[0m cc  \u{1b}[2md\u{1b}[0m\n"
    );
}

#[test]
fn a_list_without_marks_keeps_its_rows_flush() {
    let text = COLOR.list("U", None, &[], &[Row::cells(Mark::Blank, &["nico"])]);

    assert_eq!(text, "\u{1b}[1mU\u{1b}[0m\n\n  nico\n");
}

#[test]
fn a_header_row_lines_up_over_marked_rows() {
    let plain = PLAIN.list(
        "U",
        None,
        &["USER", "SHELL"],
        &[Row::cells(Mark::On, &["nico", "bash"])],
    );

    let color = COLOR.list(
        "U",
        None,
        &["USER", "SHELL"],
        &[Row::cells(Mark::On, &["nico", "bash"])],
    );

    assert_eq!(plain, "U\n\n  USER  SHELL\n  nico  bash\n");
    assert_eq!(
        color,
        "\u{1b}[1mU\u{1b}[0m\n\n    \u{1b}[2mUSER  SHELL\u{1b}[0m\n  \u{1b}[32m●\u{1b}[0m nico  \u{1b}[2mbash\u{1b}[0m\n"
    );
}

#[test]
fn every_mark_has_its_own_sign() {
    let text = COLOR.list(
        "M",
        None,
        &[],
        &[
            Row::cells(Mark::Unknown, &["a"]),
            Row::cells(Mark::Problem, &["b"]),
        ],
    );

    assert!(text.contains("\u{1b}[33m?\u{1b}[0m a"), "{text}");
    assert!(text.contains("\u{1b}[31m✗\u{1b}[0m b"), "{text}");
}

#[test]
fn an_empty_list_says_so_under_its_title() {
    assert_eq!(
        PLAIN.list("Services", Some("0"), &[], &[]),
        "Services  (0)\n\n  none\n"
    );
}

#[test]
fn facts_line_their_values_up_under_the_title() {
    let text = PLAIN.facts("Network", &[("router", "192.168.1.1"), ("wifi", "on")]);

    assert_eq!(text, "Network\n\n  router  192.168.1.1\n  wifi    on\n");
}

#[test]
fn coloured_facts_dim_their_names() {
    let text = COLOR.facts("N", &[("a", "b")]);

    assert_eq!(text, "\u{1b}[1mN\u{1b}[0m\n\n  \u{1b}[2ma\u{1b}[0m  b\n");
}

fn plain(cells: &[&str]) -> Row {
    Row::cells(Mark::Blank, cells)
}

#[test]
fn a_row_wider_than_the_terminal_cuts_its_widest_later_column() {
    let text = PLAIN.width(Some(26)).list(
        "Services",
        None,
        &[],
        &[plain(&["sshd", "the OpenSSH server daemon"])],
    );

    assert_eq!(text, "Services\n\n  sshd  the OpenSSH serve…\n");
}

#[test]
fn without_a_width_nothing_is_cut() {
    let cells = ["sshd", "the OpenSSH server daemon"];
    let text = PLAIN
        .width(None)
        .list("Services", None, &[], &[plain(&cells)]);

    assert!(text.contains("the OpenSSH server daemon"), "{text}");
}

#[test]
fn rows_that_fit_stay_whole() {
    let text = PLAIN.width(Some(80)).list(
        "S",
        None,
        &[],
        &[plain(&["sshd", "the OpenSSH server daemon"])],
    );

    assert!(text.contains("the OpenSSH server daemon"), "{text}");
}

#[test]
fn the_widest_later_column_is_cut_and_the_rest_stay_lined_up() {
    let text = PLAIN.width(Some(30)).list(
        "Files",
        None,
        &[],
        &[
            plain(&["pacman", "/etc/pacman.d/mirrorlist.pacnew", "root"]),
            plain(&["fstab", "/etc/fstab", "root"]),
        ],
    );

    assert_eq!(
        text,
        "Files\n\n  pacman  /etc/pacman.d…  root\n  fstab   /etc/fstab      root\n"
    );
}

#[test]
fn the_name_is_never_cut() {
    let name = "a-very-long-service-name-that-fills-the-line";
    let text = PLAIN
        .width(Some(20))
        .list("S", None, &[], &[plain(&[name, "the description"])]);

    assert!(text.contains(name), "{text}");
}

#[test]
fn a_column_too_narrow_to_read_is_left_whole() {
    let text = PLAIN.width(Some(30)).list(
        "S",
        None,
        &[],
        &[plain(&["a-long-service-name", "the description of it"])],
    );

    assert!(text.contains("the description of it"), "{text}");
}

#[test]
fn wide_characters_count_twice() {
    let text = PLAIN
        .width(Some(16))
        .list("S", None, &[], &[plain(&["ja", "日本語の説明文です"])]);

    assert_eq!(text, "S\n\n  ja  日本語の…\n");
}

#[test]
fn the_marks_count_toward_the_width() {
    let text = COLOR.width(Some(28)).list(
        "S",
        None,
        &[],
        &[Row::cells(Mark::On, &["sshd", "the OpenSSH server daemon"])],
    );

    assert!(text.contains("the OpenSSH serve…"), "{text}");
}

#[test]
fn the_head_is_cut_with_its_column() {
    let text = PLAIN.width(Some(26)).list(
        "Pages",
        None,
        &["PAGE", "A VERY LONG HEADING NAME HERE"],
        &[plain(&["home", "the OpenSSH server daemon"])],
    );

    assert!(text.contains("  PAGE  A VERY LONG HEADI…\n"), "{text}");
}

#[test]
fn dim_text_is_dimmed_on_a_terminal_only() {
    assert_eq!(PLAIN.dim("(4.2 s)"), "(4.2 s)");
    assert_eq!(
        Paint::new(Style::Color).dim("(4.2 s)"),
        "\u{1b}[2m(4.2 s)\u{1b}[0m"
    );
}

#[test]
fn a_row_takes_its_cells_as_borrowed_text() {
    assert_eq!(
        Row::cells(Mark::On, &["sshd", "running"]),
        Row::new(Mark::On, vec!["sshd".to_owned(), "running".to_owned()])
    );
}

/// A package description comes from its packager: an escape sequence in
/// it must not reach the terminal.
#[test]
fn text_from_outside_loses_its_escapes() {
    let rows = [Row::cells(
        Mark::Blank,
        &["evil", "nice\u{1b}]52;c;cHduZWQ=\u{7} tool"],
    )];

    let listed = Paint::new(Style::Plain).list("Packages", None, &[], &rows);
    let facts = Paint::new(Style::Plain).facts("Package", &[("Description", "a\u{1b}[2Jb")]);

    assert!(!listed.contains('\u{1b}'), "{listed:?}");
    assert!(!facts.contains('\u{1b}'), "{facts:?}");
}
