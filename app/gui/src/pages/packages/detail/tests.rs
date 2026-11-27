mod below;
mod libraries;
mod lines;

use egui::accesskit::Role;
use egui::{Color32, Key};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::packages::OptionalDependency;
use toolbox_core::features::packages::history::{HistoryAction, HistoryEntry};
use toolbox_gui_kit::fonts::{self, PAGE_TITLE, Weight};
use toolbox_gui_kit::testing::filled_rects;
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{MIN_TEXT_WIDTH, hue_of, icon_button_width};

use super::super::PackagesPage;
use super::super::tests::{fixture, gimp_details, listed, name};
use crate::command::{Command, PackagesCommand};
use crate::packages::{Details, Read};
use crate::testing::{PageState, rect_of};

fn gimp_page() -> PackagesPage {
    PackagesPage {
        open: Some(name("gimp")),
        ..PackagesPage::default()
    }
}

fn showing(details: Details) -> Harness<'static, PageState<PackagesPage>> {
    let mut harness = fixture(Some(details)).harness(gimp_page());
    harness.run();
    harness
}

fn gimp_with(edit: impl FnOnce(&mut toolbox_core::features::packages::PackageDetails)) -> Details {
    let mut details = gimp_details();
    if let Read::Known(Some(info)) = &mut details.info {
        edit(info);
    }

    details
}

fn expanded<S>(harness: &Harness<'_, S>, label: &str) -> Option<bool> {
    harness
        .get_by_role_and_label(Role::Button, label)
        .accesskit_node()
        .data()
        .is_expanded()
}

fn change(day: u8) -> HistoryEntry {
    HistoryEntry {
        when: format!("2025-06-{day:02}T08:15:00+0200"),
        action: HistoryAction::Upgraded,
        versions: format!("1.{day}-1 -> 1.{}-1", day + 1),
    }
}

#[test]
fn one_page_shows_the_facts_above_what_the_package_depends_on() {
    let harness = showing(gimp_details());

    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Depends on")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Required by")
            .is_some()
    );
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Optional dependencies")
            .is_some()
    );
    assert!(harness.query_by_label("Installed explicitly").is_some());
    assert!(
        harness.query_by_role(Role::RadioButton).is_none(),
        "no tabs"
    );

    let facts = rect_of(&harness, Role::Label, "Repository");
    let depends = rect_of(&harness, Role::Heading, "Depends on");
    assert!(depends.top() > facts.bottom(), "stacked under the facts");
}

#[test]
fn the_web_address_is_a_link_without_its_scheme() {
    let harness = showing(gimp_details());

    let link = harness.get_by_role_and_label(Role::Link, "https://www.gimp.org/");
    assert!(link.accesskit_node().data().value().is_none());
    assert!(harness.query_by_label("Copy Website").is_some());
}

#[test]
fn a_dependency_opens_its_package_and_back_walks_the_chain_to_the_list() {
    let mut harness = showing(gimp_details());

    harness
        .get_by_role_and_label(Role::Link, "gegl>=0.4")
        .click();
    harness.run();
    assert_eq!(harness.state().page.open, Some(name("gegl")));
    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(PackagesCommand::Show(name("gegl")))]
    );

    harness.get_by_label("Back to gimp").click();
    harness.run();
    assert_eq!(harness.state().page.open, Some(name("gimp")));

    // Back would only lead to the list here, so Escape leaves the details instead.
    harness.key_press(Key::Escape);
    harness.run();
    harness.run();
    assert_eq!(harness.state().page.open, None);
    assert!(listed(&harness, "gimp"));
}

#[test]
fn a_name_no_package_answers_to_is_plain_text() {
    let harness = showing(gimp_with(|info| {
        info.depends = vec!["foo-utils>=1".to_owned()];
    }));

    assert!(harness.query_by_label("foo-utils>=1").is_some());
    assert!(
        harness
            .query_by_role_and_label(Role::Link, "foo-utils>=1")
            .is_none()
    );
}

#[test]
fn a_long_list_shows_twelve_names_and_the_rest_in_place() {
    let many: Vec<String> = (1_u8..=15).map(|place| format!("lib{place}")).collect();
    let mut harness = showing(gimp_with(|info| info.depends = many));

    assert!(harness.query_by_label("lib12").is_some());
    assert!(harness.query_by_label("lib13").is_none());

    harness
        .get_by_role_and_label(Role::Button, "Show 3 more")
        .click();
    harness.run();

    assert!(harness.query_by_label("lib15").is_some());
    assert!(harness.query_by_label("Show 3 more").is_none());
}

#[test]
fn an_optional_dependency_names_what_it_adds() {
    let optional = vec![OptionalDependency {
        name: toolbox_plan::PackageName::new("gegl").unwrap(),
        reason: toolbox_text::plain("for filters"),
    }];

    let harness = showing(gimp_with(|info| info.optional = optional));

    assert!(
        harness
            .query_by_role_and_label(Role::Link, "gegl")
            .is_some()
    );

    assert!(harness.query_by_label(": for filters").is_some());
}

#[test]
fn the_files_start_folded_and_copy_list_copies_them_all() {
    let mut harness = showing(gimp_details());

    assert_eq!(expanded(&harness, "Files 1"), Some(false));
    assert!(harness.query_by_label("/usr/bin/gimp").is_none());

    harness
        .get_by_role_and_label(Role::Button, "Copy List")
        .click();
    harness.step();
    let copied = harness.output().platform_output.commands.iter().any(
        |command| matches!(command, egui::OutputCommand::CopyText(text) if text == "/usr/bin/gimp\n"),
    );

    harness.run();

    assert!(copied);

    harness
        .get_by_role_and_label(Role::Button, "Files 1")
        .click();
    harness.run();
    assert_eq!(expanded(&harness, "Files 1"), Some(true));
    assert!(harness.query_by_label("/usr/bin/gimp").is_some());
}

#[test]
fn a_package_that_is_not_installed_has_no_files() {
    let harness = showing(gimp_with(|info| info.installed = None));

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "Files 1")
            .is_none()
    );
    assert!(harness.query_by_label("Installed explicitly").is_none());
}

#[test]
fn the_history_opens_with_its_newest_five_lines_and_shows_all_on_request() {
    let mut details = gimp_details();
    details.history = Read::Known((1..=7).rev().map(change).collect());
    let mut harness = showing(details);

    // The Show All button sits below the fold of the window at the usual height.
    harness.set_size(egui::vec2(1024.0, 1600.0));
    harness.run();
    assert_eq!(expanded(&harness, "History"), Some(true));
    assert!(harness.query_by_label("Upgraded 1.7-1 → 1.8-1").is_some());
    assert!(harness.query_by_label("Upgraded 1.3-1 → 1.4-1").is_some());
    assert!(harness.query_by_label("Upgraded 1.2-1 → 1.3-1").is_none());

    harness
        .get_by_role_and_label(Role::Button, "Show All")
        .click();
    harness.run();

    assert!(harness.query_by_label("Upgraded 1.1-1 → 1.2-1").is_some());
    assert!(harness.query_by_label("Show All").is_none());
}

#[test]
fn a_short_history_has_no_show_all() {
    let mut details = gimp_details();
    details.history = Read::Known(vec![change(1)]);
    let harness = showing(details);

    assert!(harness.query_by_label("Upgraded 1.1-1 → 1.2-1").is_some());
    assert!(harness.query_by_label("Show All").is_none());
}

/// pacman has one description: it is the subtitle under the name, so the
/// left column starts with the facts.
#[test]
fn the_description_shows_once_under_the_name() {
    let harness = showing(gimp_details());

    assert_eq!(harness.query_all_by_label("About gimp").count(), 1);
    assert!(
        harness
            .query_by_role_and_label(Role::Heading, "Description")
            .is_none()
    );
}

#[test]
fn the_letter_tile_is_as_tall_as_the_title_line_from_its_capitals() {
    let harness = showing(gimp_details());

    let title = rect_of(&harness, Role::Heading, "gimp");
    let font = fonts::font(&harness.ctx.global_style(), Weight::Regular, PAGE_TITLE);
    let capital = harness
        .ctx
        .fonts_mut(|fonts| fonts.layout_no_wrap("H".to_owned(), font, Color32::PLACEHOLDER));

    let fill = tokens::current(&harness.ctx)
        .tiles
        .letters()
        .of(hue_of("gimp"))
        .fill;

    // The list shows the same tile beside its row, so the title's is the one
    // nearest to the title.
    let picture = filled_rects(&harness, fill)
        .into_iter()
        .min_by(|first, second| {
            let near = |rect: &egui::Rect| rect.center().distance(title.center());
            near(first).total_cmp(&near(second))
        })
        .expect("gimp has a letter tile");

    assert!((picture.height() - capital.rect.height()).abs() < 0.5);
    assert!(
        (picture.top() - (title.top() + capital.mesh_bounds.top())).abs() < 0.5,
        "tile {picture:?}, title {title:?}, capitals at {}",
        capital.mesh_bounds.top()
    );
}

/// On every narrow width the back button keeps its text only while the title
/// still has its least room beside the controls.
#[test]
fn the_back_buttons_text_never_squeezes_the_title() {
    let mut harness = showing(gimp_details());
    let gap = tokens::current(&harness.ctx).sizes.text_to_control;

    let mut texts = 0_u32;
    for width in (400_u16..640).step_by(2) {
        harness.set_size(egui::vec2(f32::from(width), 480.0));
        harness.run();

        let Some(back) = harness.query_by_role_and_label(Role::Button, "Back to Packages") else {
            continue;
        };

        let back = back.rect();
        if back.width() <= icon_button_width() + 0.5 {
            continue;
        }

        texts += 1;
        let title = rect_of(&harness, Role::Heading, "gimp");
        let remove = rect_of(&harness, Role::Button, "Remove gimp");
        let room = remove.left() - gap - title.left();
        assert!(
            room >= MIN_TEXT_WIDTH - 0.5,
            "at {width}: the title has {room} beside {back:?} and {remove:?}"
        );
    }

    assert!(texts > 0, "the back button never shows its text");
    harness.set_size(egui::vec2(400.0, 480.0));
    harness.run();
    let back = rect_of(&harness, Role::Button, "Back to Packages");
    assert!(
        back.width() <= icon_button_width() + 0.5,
        "too narrow for the text"
    );
}
