use egui::accesskit::Role;
use egui::{CentralPanel, Event, Frame, Theme};
use egui_kittest::kittest::Queryable as _;
use egui_kittest::{Harness, SnapshotResults};
use toolbox_gui_kit::testing::{filled_rects, painted_text};
use toolbox_guide::{Block, Code};

use super::super::markdown::{Reader, blocks};

/// The command `boot-options.md` shows, and a line it could print.
const CONSOLE: &str = "$ garuda-toolbox boot param add systemd.unit=rescue.target --force\nAdded systemd.unit=rescue.target.";
const JSON: &str = "{\n  \"update-check\": 30,\n  \"start-at-login\": true\n}";
const ONE_LINE: &str = "$ garuda-toolbox status";

fn code(language: &str, text: &str) -> Block {
    Block::Code(Code {
        language: Some(language.to_owned()),
        text: text.to_owned(),
    })
}

fn shown(theme: Theme, shown: Vec<Block>) -> Harness<'static, bool> {
    let mut harness = toolbox_gui_kit::testing::harness(640.0, 260.0)
        .with_theme(theme)
        .build_ui_state(
            move |ui, copied: &mut bool| {
                let window = toolbox_gui_kit::tokens::current(ui.ctx()).window;

                // The harness leaves a clear margin round the panel, which
                // the window's fill covers in the app.
                let screen = ui.ctx().content_rect();
                ui.ctx()
                    .layer_painter(egui::LayerId::background())
                    .rect_filled(screen, 0.0, window);
                CentralPanel::default()
                    .frame(Frame::new().fill(window).inner_margin(16))
                    .show(ui, |ui| {
                        let mut reader = Reader::new(None);
                        blocks(ui, &shown, &mut reader);
                        *copied |= reader.copied;
                    });
            },
            false,
        );

    harness.run();
    harness
}

fn both(theme: Theme) -> Harness<'static, bool> {
    shown(theme, vec![code("console", CONSOLE), code("json", JSON)])
}

/// The code blocks the frame painted, the copy buttons' backings left out.
fn block_rects(harness: &Harness<'_, bool>) -> Vec<egui::Rect> {
    let field = toolbox_gui_kit::tokens::current(&harness.ctx).field;
    filled_rects(harness, field)
        .into_iter()
        .filter(|rect| rect.width() > 100.0)
        .collect()
}

fn hover_first(harness: &mut Harness<'_, bool>) {
    let first = block_rects(harness)[0];
    harness.event(Event::PointerMoved(first.left_center()));
    harness.run();
}

#[test]
fn code_blocks_at_rest() {
    let mut results = SnapshotResults::new();
    for (theme, name) in [(Theme::Dark, "dark"), (Theme::Light, "light")] {
        let mut harness = both(theme);
        results.add(harness.try_snapshot(format!("guide/code_blocks_{name}")));
    }
}

#[test]
fn code_blocks_with_the_pointer_on_one() {
    let mut results = SnapshotResults::new();
    for (theme, name) in [(Theme::Dark, "dark"), (Theme::Light, "light")] {
        let mut harness = both(theme);
        hover_first(&mut harness);
        results.add(harness.try_snapshot(format!("guide/code_blocks_hovered_{name}")));
    }
}

#[test]
fn a_one_line_block_is_as_tall_as_its_line_and_padding() {
    let harness = shown(Theme::Dark, vec![code("console", ONE_LINE)]);

    let line = painted_text(&harness, ONE_LINE)
        .expect("the line is painted")
        .galley
        .rect
        .height();

    let block = block_rects(&harness)[0];

    assert!(
        (block.height() - (line + 24.0)).abs() < 0.5,
        "block {block:?}, line {line}"
    );
}

#[test]
fn the_copy_button_sits_in_the_blocks_top_right_corner() {
    let harness = shown(Theme::Dark, vec![code("console", CONSOLE)]);

    let block = block_rects(&harness)[0];
    let button = harness
        .get_by_role_and_label(Role::Button, "Copy Code")
        .rect();

    assert!(block.contains_rect(button), "{button:?} in {block:?}");
    assert!(
        (block.right() - button.right() - (button.top() - block.top())).abs() < 0.5,
        "as far from the right edge as from the top: {button:?} in {block:?}"
    );
}

#[test]
fn a_hovered_block_copies_its_text_and_says_so() {
    let mut harness = both(Theme::Dark);

    hover_first(&mut harness);
    harness
        .query_all_by_role_and_label(Role::Button, "Copy Code")
        .next()
        .expect("a copy button")
        .click();
    harness.step();

    let copied =
        harness.output().platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if text == CONSOLE),
        );

    assert!(copied, "the block's text is on the clipboard");
    assert!(*harness.state(), "the page hears of the copy");
}
