mod lines;

use egui::accesskit::Role;
use egui::{Color32, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use toolbox_core::features::packages::OptionalDependency;
use toolbox_core::features::packages::history::{HistoryAction, HistoryEntry};
use toolbox_gui_kit::fonts::{self, TITLE, Weight};
use toolbox_gui_kit::testing::filled_rect;
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::hue_of;

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

/// gimp with `edit` made to its details.
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
fn one_page_shows_the_facts_beside_what_the_package_depends_on() {
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
    assert!(depends.left() > facts.right(), "side by side");
}

#[test]
fn a_narrow_page_stacks_the_dependencies_under_the_facts() {
    let mut harness = showing(gimp_details());
    harness.set_size(vec2(560.0, 720.0));
    harness.run();

    let facts = rect_of(&harness, Role::Label, "Repository");
    let depends = rect_of(&harness, Role::Heading, "Depends on");
    assert!(depends.top() > facts.bottom(), "stacked");
    assert!((depends.left() - facts.left()).abs() < 1.0);
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

    harness.get_by_label("Back to Packages").click();
    harness.run();
    harness.run();
    assert_eq!(harness.state().page.open, None);
    assert!(listed(&harness, "gimp"));
}

#[test]
fn a_name_no_package_answers_to_is_plain_text() {
    let harness = showing(gimp_with(|info| {
        info.depends = vec!["libfoo.so=1-64".to_owned()];
    }));

    assert!(harness.query_by_label("libfoo.so=1-64").is_some());
    assert!(
        harness
            .query_by_role_and_label(Role::Link, "libfoo.so=1-64")
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
        name: "gegl".to_owned(),
        reason: "for filters".to_owned(),
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
        .get_by_role_and_label(Role::Button, "Copy list")
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
    assert!(listed(&harness, "/usr/bin/gimp"));
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

    assert_eq!(expanded(&harness, "History"), Some(true));
    assert!(harness.query_by_label("Upgraded 1.7-1 → 1.8-1").is_some());
    assert!(harness.query_by_label("Upgraded 1.3-1 → 1.4-1").is_some());
    assert!(harness.query_by_label("Upgraded 1.2-1 → 1.3-1").is_none());

    harness
        .get_by_role_and_label(Role::Button, "Show all")
        .click();
    harness.run();

    assert!(harness.query_by_label("Upgraded 1.1-1 → 1.2-1").is_some());
    assert!(harness.query_by_label("Show all").is_none());
}

#[test]
fn a_short_history_has_no_show_all() {
    let mut details = gimp_details();
    details.history = Read::Known(vec![change(1)]);
    let harness = showing(details);

    assert!(harness.query_by_label("Upgraded 1.1-1 → 1.2-1").is_some());
    assert!(harness.query_by_label("Show all").is_none());
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
fn both_columns_start_on_one_line() {
    let harness = showing(gimp_details());

    let facts = rect_of(&harness, Role::Label, "Repository");
    let depends = rect_of(&harness, Role::Heading, "Depends on");
    assert!(
        (depends.top() - facts.top()).abs() < 0.5,
        "the facts start at {}, the dependencies at {}",
        facts.top(),
        depends.top()
    );
}

#[test]
fn the_letter_tile_is_as_tall_as_the_title_line_from_its_capitals() {
    let harness = showing(gimp_details());

    let title = rect_of(&harness, Role::Heading, "gimp");
    let font = fonts::font(&harness.ctx.global_style(), Weight::Bold, TITLE);
    let capital = harness
        .ctx
        .fonts_mut(|fonts| fonts.layout_no_wrap("H".to_owned(), font, Color32::PLACEHOLDER));

    let fill = tokens::current(&harness.ctx)
        .tiles
        .letters()
        .of(hue_of("gimp"))
        .fill;

    let picture = filled_rect(&harness, fill).expect("gimp has a letter tile");

    assert!((picture.height() - capital.rect.height()).abs() < 0.5);
    assert!((picture.top() - (title.top() + capital.mesh_bounds.top())).abs() < 0.5);
}
