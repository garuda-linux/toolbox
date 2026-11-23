//! A code block: its copy button sits at the block's right edge, however
//! short the code is.

use egui_kittest::kittest::Queryable as _;

use super::page_on;

const FIRST_CONSOLE: &str = "$ garuda-toolbox\n$ garuda-toolbox --page maintenance\n$ garuda-toolbox status\n$ garuda-toolbox update --yes";

#[test]
fn every_copy_button_sits_at_the_right_edge() {
    let mut harness = page_on("cli.md");
    harness.set_size(egui::vec2(1024.0, 4000.0));
    harness.run();

    let rights: Vec<f32> = harness
        .query_all_by_label("Copy Code")
        .map(|node| node.rect().right())
        .collect();

    assert!(rights.len() > 1, "cli.md shows fewer than two code blocks");
    let widest = rights.iter().copied().fold(f32::MIN, f32::max);
    for right in &rights {
        assert!(
            (widest - right).abs() < 1.0,
            "a copy button ends at {right}, the rightmost at {widest}"
        );
    }
}

fn inks(harness: &super::Page, text: &str) -> Vec<(String, egui::Color32)> {
    let painted = toolbox_gui_kit::testing::painted_text(harness, text)
        .unwrap_or_else(|| panic!("{text:?} is not painted"));

    let job = &painted.galley.job;
    job.sections
        .iter()
        .map(|section| {
            let range = section.byte_range.start.0..section.byte_range.end.0;
            (
                job.text.get(range).unwrap_or_default().to_owned(),
                section.format.color,
            )
        })
        .collect()
}

#[test]
fn a_console_block_dims_its_prompts_and_keeps_its_text() {
    let mut harness = page_on("cli.md");
    harness.set_size(egui::vec2(1024.0, 4000.0));
    harness.run();

    let code = toolbox_gui_kit::tokens::current(&harness.ctx).code;
    let inks = inks(&harness, FIRST_CONSOLE);

    let prompts: Vec<_> = inks.iter().filter(|(text, _)| text == "$ ").collect();
    assert_eq!(prompts.len(), 4, "{inks:?}");
    assert!(prompts.iter().all(|(_, ink)| *ink == code.dim));
    assert!(
        inks.iter()
            .any(|(text, ink)| text.trim() == "status" && *ink == code.plain),
        "{inks:?}"
    );
    assert!(
        harness.query_by_label(FIRST_CONSOLE).is_some(),
        "screen readers hear the code as written"
    );
}

#[test]
fn a_command_colours_its_program_flags_and_values() {
    let mut harness = page_on("boot-options.md");
    harness.set_size(egui::vec2(1024.0, 4000.0));
    harness.run();

    let code = toolbox_gui_kit::tokens::current(&harness.ctx).code;
    let inks = inks(
        &harness,
        "$ garuda-toolbox boot param add systemd.unit=rescue.target --force",
    );

    // The layout joins neighbours of one ink into one section.
    let ink_of = |part: &str| {
        inks.iter()
            .find(|(text, _)| text.contains(part))
            .map(|(_, ink)| *ink)
    };

    assert_eq!(ink_of("garuda-toolbox"), Some(code.function), "{inks:?}");
    assert_eq!(ink_of("param"), Some(code.plain), "{inks:?}");
    assert_eq!(ink_of("systemd.unit="), Some(code.plain), "{inks:?}");
    assert_eq!(ink_of("rescue.target"), Some(code.string), "{inks:?}");
    assert_eq!(ink_of("--force"), Some(code.keyword), "{inks:?}");
}
