mod ground;

use egui::accesskit::{Live, Role};
use egui::{Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_terminal::{Terminal, TerminalSize, encode_paste};

use super::{TerminalOutput, TerminalView};
use crate::testing::harness;

const HINT: &str = "Ctrl+Tab leaves the terminal.";

/// What `script(1)` caught of pacman asking for a `java-runtime` provider
/// and then whether to install.
const PACMAN_PROVIDER: &[u8] = toolbox_pty::testing::PACMAN_PROVIDER_TXT.as_bytes();

struct Console {
    terminal: Terminal,
    bytes: Vec<u8>,
    fitting: Option<TerminalSize>,
}

fn console(text: &[u8]) -> Harness<'static, Console> {
    let mut terminal = Terminal::new(TerminalSize {
        columns: 40,
        rows: 6,
    });

    terminal.feed(text);

    let state = Console {
        terminal,
        bytes: Vec::new(),
        fitting: None,
    };

    // A terminal shows whatever programs print, and the bundled fonts lack
    // CJK glyphs. A box still shows where a wide character sits.
    let mut harness = harness(500.0, 200.0).allow_missing_glyphs().build_ui_state(
        |ui, console: &mut Console| {
            let TerminalOutput {
                response: _,
                bytes,
                fitting,
            } = TerminalView::new(&mut console.terminal, "Console")
                .hint(HINT)
                .show(ui);

            console.bytes.extend(bytes);
            console.fitting = Some(fitting);
        },
        state,
    );

    harness.run();
    harness
}

fn focus(harness: &mut Harness<'_, Console>) {
    harness
        .get_by_role_and_label(Role::Terminal, "Console")
        .click();
    harness.run();
}

fn release(harness: &mut Harness<'_, Console>, pos: egui::Pos2) {
    harness.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    harness.run();
}

#[test]
fn the_terminal_is_named_for_screen_readers() {
    let harness = console(b"");

    assert!(
        harness
            .query_by_role_and_label(Role::Terminal, "Console")
            .is_some()
    );
}

#[test]
fn typed_text_goes_to_the_program_once_focused() {
    let mut harness = console(b"$ ");
    focus(&mut harness);

    harness
        .get_by_role_and_label(Role::Terminal, "Console")
        .type_text("ls");
    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(harness.state().bytes, b"ls\r");
}

#[test]
fn tab_and_escape_stay_with_the_program() {
    let mut harness = console(b"");
    focus(&mut harness);

    harness.key_press(Key::Tab);
    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().bytes, b"\t\x1b");
}

fn ctrl_shift_v(harness: &mut Harness<'_, Console>) {
    harness.key_press_modifiers(
        Modifiers {
            ctrl: true,
            shift: true,
            command: true,
            ..Modifiers::NONE
        },
        Key::V,
    );
    harness.step();
}

/// eframe answers the request with the clipboard as a paste event in the
/// next frame, which kittest leaves to the test.
#[test]
fn ctrl_shift_v_pastes_the_clipboard_it_asked_for() {
    let mut harness = console(b"");
    focus(&mut harness);

    ctrl_shift_v(&mut harness);
    harness.event(egui::Event::Paste("echo hi\n".to_owned()));
    harness.step();

    let mode = harness.state().terminal.key_mode();
    assert_eq!(harness.state().bytes, encode_paste("echo hi\n", mode));
}

/// kittest counts a quarter second per frame, so five frames outlast the ask.
#[test]
fn a_paste_after_the_ask_ran_out_stays_ctrl_v() {
    let mut harness = console(b"");
    focus(&mut harness);

    ctrl_shift_v(&mut harness);
    for _ in 0_u8..5 {
        harness.step();
    }

    harness.event(egui::Event::Paste("echo hi\n".to_owned()));
    harness.run();

    assert_eq!(harness.state().bytes, b"\x16");
}

#[test]
fn the_fitting_size_follows_the_widget() {
    let harness = console(b"");
    let fitting = harness.state().fitting.unwrap();

    assert!(fitting.columns >= 40, "{fitting:?}");
    assert!(fitting.rows >= 6, "{fitting:?}");
}

#[test]
fn a_focused_terminal_owns_the_keys() {
    let mut harness = console(b"");
    assert!(!super::owns_keys(&harness.ctx));

    focus(&mut harness);

    assert!(super::owns_keys(&harness.ctx));
}

#[test]
fn nothing_is_sent_without_focus() {
    let mut harness = console(b"");

    harness.key_press(Key::Enter);
    harness.run();

    assert_eq!(harness.state().bytes, b"");
}

#[test]
fn a_coloured_screen_with_a_selection() {
    let mut harness = console(
        b"\x1b[1;32m==>\x1b[0m Installing \x1b[4mfirefox\x1b[0m\r\n\
          \x1b[31merror:\x1b[0m \x1b[2mfaint\x1b[0m \x1b[7m inverse \x1b[0m\r\n\
          wide: \xe5\xae\x89\xe8\xa3\x85 ok\r\n\
          \x1b[38;5;208m256\x1b[0m \x1b[38;2;10;200;250mtrue\x1b[0m\r\n$ ",
    );

    let rect = harness
        .get_by_role_and_label(Role::Terminal, "Console")
        .rect();

    let end = rect.min + egui::vec2(150.0, 22.0);

    harness.drag_at(rect.min + egui::vec2(2.0, 2.0));
    harness.run();
    harness.hover_at(end);
    harness.run();
    release(&mut harness, end);

    harness.snapshot("terminal/colours_dark");
}

#[test]
fn a_drag_over_two_rows_copies_both_with_ctrl_shift_c() {
    let mut harness = console(b"first line here\r\nsecond line here\r\n");
    focus(&mut harness);
    let rect = harness
        .get_by_role_and_label(Role::Terminal, "Console")
        .rect();

    let row = rect.height() / 6.0;
    let end = rect.min + egui::vec2(40.0, row * 1.5);

    harness.drag_at(rect.min + egui::vec2(1.0, row * 0.5));
    harness.run();
    harness.hover_at(end);
    harness.run();
    release(&mut harness, end);

    // The press alone: kittest runs one frame per queued event and keeps
    // the newest frame's output only.
    harness.event(egui::Event::Key {
        key: Key::C,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers {
            ctrl: true,
            shift: true,
            command: true,
            ..Modifiers::NONE
        },
    });
    harness.step();

    let copied: Vec<String> = harness
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|command| {
            if let egui::OutputCommand::CopyText(text) = command {
                Some(text.clone())
            } else {
                None
            }
        })
        .collect();

    assert_eq!(copied.len(), 1, "{copied:?}");
    assert!(copied[0].starts_with("first line here\nsec"), "{copied:?}");
}

#[test]
fn a_screen_reader_reads_the_text_on_screen_as_it_comes() {
    let harness = console(PACMAN_PROVIDER);
    let terminal = harness.get_by_role_and_label(Role::Terminal, "Console");
    let node = terminal.accesskit_node();
    let value = node.value().unwrap_or_default();

    assert!(
        value.ends_with("\n:: Proceed with installation? [Y/n]"),
        "{value:?}"
    );
    assert!(!value.contains('\u{1b}'), "{value:?}");
    assert_eq!(node.live(), Live::Polite);
}

#[test]
fn the_terminal_tells_how_to_leave_it() {
    let harness = console(b"");
    let terminal = harness.get_by_role_and_label(Role::Terminal, "Console");

    assert_eq!(
        terminal.accesskit_node().description().as_deref(),
        Some(HINT)
    );
}

#[test]
fn plain_output_is_no_live_region_so_progress_does_not_flood_the_reader() {
    let harness = console(b"downloading 42%\r\ndownloading 43%");
    let terminal = harness.get_by_role_and_label(Role::Terminal, "Console");

    assert_eq!(terminal.accesskit_node().live(), Live::Off);
}
