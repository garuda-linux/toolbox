mod callout;
mod contents;
mod picture;

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use parking_lot::Mutex;

use super::GuidePage;
use crate::command::{Command, UiCommand};
use crate::links::{Opener, install};
use crate::page::PageId;
use crate::testing::{PageFixture, PageState};

#[derive(Debug, Default)]
struct Opened(Mutex<Vec<String>>);

impl Opener for Opened {
    fn open(&self, url: &str) -> bool {
        self.0.lock().push(url.to_owned());
        true
    }
}

type Page = Harness<'static, PageState<GuidePage>>;

fn page() -> Page {
    let mut harness = PageFixture::default().harness(GuidePage::default());
    harness.run();
    harness
}

fn page_on(file: &str) -> Page {
    let mut harness = page();
    harness.state_mut().page.show(file, None);
    harness.run();
    harness
}

fn heading_shows(harness: &Page, text: &str) -> bool {
    harness
        .query_all_by_role(Role::Heading)
        .any(|node| node.accesskit_node().label().as_deref() == Some(text))
}

fn sent(harness: &Page) -> Vec<Command> {
    harness.state().fixture.sent()
}

#[test]
fn it_opens_on_the_introduction_with_every_page_listed() {
    let harness = page();

    assert!(heading_shows(&harness, "Introduction"));
    for title in ["Kernels", "Troubleshooting", "User guide"] {
        assert!(
            harness
                .query_all_by_role_and_label(Role::Link, title)
                .next()
                .is_some(),
            "{title} is not in the list"
        );
    }
}

#[test]
fn a_page_in_the_list_opens_in_the_reader() {
    let mut harness = page();

    harness.get_by_role_and_label(Role::Link, "Kernels").click();
    harness.run();

    assert!(heading_shows(&harness, "Kernels"));
    assert!(heading_shows(&harness, "Install a kernel"));
}

#[test]
fn a_link_to_another_page_opens_it_in_the_reader() {
    let mut harness = page();

    let label = "Review and apply changes";
    let link = |shown_in: &Page| {
        shown_in
            .query_all_by_role_and_label(Role::Link, label)
            .last()
            .expect("the introduction links to the page")
            .rect()
    };

    let shown = link(&harness);
    harness
        .query_all_by_role_and_label(Role::Link, label)
        .last()
        .expect("the introduction links to the page")
        .scroll_to_me();
    harness.run();
    assert!(link(&harness).top() <= shown.top());
    harness
        .query_all_by_role_and_label(Role::Link, label)
        .last()
        .expect("the introduction links to the page")
        .click();
    harness.run();

    assert!(heading_shows(&harness, "Review and apply changes"));
}

#[test]
fn the_open_button_opens_the_page_the_guide_page_is_about() {
    let mut harness = page_on("kernels.md");

    harness
        .get_by_role_and_label(Role::Button, "Open Kernels")
        .click();
    harness.run();

    assert_eq!(
        sent(&harness),
        [Command::Ui(UiCommand::Navigate(PageId::Kernels))]
    );
}

#[test]
fn a_guide_page_about_no_app_page_has_no_open_button() {
    let harness = page_on("troubleshooting.md");

    let opens = harness
        .query_all_by_role(Role::Button)
        .filter(|node| {
            node.accesskit_node()
                .label()
                .is_some_and(|label| label.starts_with("Open "))
        })
        .count();

    assert_eq!(opens, 0);
}

#[test]
fn a_web_link_opens_in_the_browser() {
    let mut harness = page_on("console.md");
    let opened = Arc::new(Opened::default());
    let fake: Arc<dyn Opener> = Arc::<Opened>::clone(&opened);
    install(&harness.ctx, fake);

    harness
        .get_by_role_and_label(Role::Link, "issue tracker")
        .scroll_to_me();
    harness.run();
    harness
        .get_by_role_and_label(Role::Link, "issue tracker")
        .click();
    harness.run();

    for _ in 0..200_u16 {
        if !opened.0.lock().is_empty() {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    assert_eq!(
        *opened.0.lock(),
        ["https://gitlab.com/garuda-linux/applications/toolbox/-/issues"]
    );
}

#[test]
fn a_search_lists_the_places_and_one_opens_at_its_heading() {
    let mut harness = page();

    "dkms headers".clone_into(&mut harness.state_mut().page.search);
    harness.run();
    harness
        .query_all_by_label_contains("Kernels › ")
        .next()
        .expect("a hit under a heading of the Kernels page")
        .click();
    harness.run();

    assert!(heading_shows(&harness, "Kernels"));
    assert_eq!(harness.state().page.search, "");
    assert_eq!(harness.state().page.file, "kernels.md");
}

#[test]
fn a_search_without_hits_says_so() {
    let mut harness = page();

    "qwertzuiop".clone_into(&mut harness.state_mut().page.search);
    harness.run();

    assert!(
        harness
            .query_by_label_contains("Nothing in the guide matches")
            .is_some()
    );
}

#[test]
fn on_this_page_shows_beside_a_wide_reader_only() {
    let mut harness = page_on("kernels.md");
    assert!(harness.query_by_label("On this page").is_none());

    harness.set_size(egui::vec2(1440.0, 900.0));
    harness.run();
    harness.run();

    assert!(harness.query_by_label("On this page").is_some());
}

#[test]
fn every_app_page_has_its_topic_in_the_guide() {
    let guide = toolbox_guide::embedded();

    for page in PageId::ALL {
        let (file, anchor) = page.guide_topic();
        let shown = guide
            .page(file)
            .unwrap_or_else(|| panic!("{file} is missing"));

        if let Some(anchor) = anchor {
            assert!(
                shown.outline.iter().any(|heading| heading.anchor == anchor),
                "{file} has no #{anchor}"
            );
        }
    }
}

#[test]
fn a_guide_page_about_an_app_page_names_it() {
    assert_eq!(PageId::of_guide_file("kernels.md"), Some(PageId::Kernels));
    assert_eq!(PageId::of_guide_file("settings.md"), Some(PageId::Settings));
    assert_eq!(PageId::of_guide_file("guide.md"), None);
    assert_eq!(PageId::of_guide_file("introduction.md"), None);
}
