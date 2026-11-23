use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_plan::Toggle;
use toolbox_system::skel::shelf::SkelRow;

use super::{kitty, open_tab, page, programs, with};
use crate::maintenance::Maintenance;

#[test]
fn a_reset_row_names_the_files_it_replaces_in_sight() {
    let mut harness = page(with(
        programs(),
        Maintenance::showing(None, false, vec![kitty()], None),
    ));

    open_tab(&mut harness, "Reset Settings");

    let row = harness.get_by_role_and_label(Role::CheckBox, "Kitty");
    assert_eq!(
        row.accesskit_node().description().as_deref(),
        Some("~/.config/kitty/kitty.conf")
    );
    assert!(row.rect().height() > 40.0, "the file line has no room");
}

#[test]
fn a_long_reset_list_scrolls_and_keeps_its_buttons_in_the_window() {
    let rows: Vec<SkelRow> = (0_u32..40)
        .map(|index| SkelRow {
            entry: format!("app{index}"),
            name: format!("App {index}"),
            files: vec![format!("/etc/skel/.config/app{index}/app.conf")],
            targets: vec![format!("~/.config/app{index}/app.conf")],
        })
        .collect();

    let mut maintenance = Maintenance::showing(None, false, rows.clone(), None);
    for row in &rows {
        maintenance.choose(&row.entry, Toggle::On);
    }

    let mut harness = page(with(programs(), maintenance));
    harness.set_size(vec2(1024.0, 600.0));
    open_tab(&mut harness, "Reset Settings");
    harness.get_by_label("Reset Selected…").scroll_to_me();
    harness.run();
    harness.get_by_label("Reset Selected…").click();
    harness.run();

    let window = harness.ctx.content_rect();
    for button in ["Cancel", "Reset"] {
        let rect = harness.get_by_role_and_label(Role::Button, button).rect();
        assert!(
            window.contains_rect(rect),
            "{button} at {rect:?} left the window {window:?}"
        );
    }
}
