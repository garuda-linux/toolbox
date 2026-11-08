use toolbox_cli::args::Window;
use toolbox_core::features::diagnostics::{Collector, Selection};
use toolbox_services::instance::PageAction;
use toolbox_services::page::PageId;

use super::{Opening, opening};

fn window(page: Option<&str>, action: Option<&str>) -> Window {
    Window {
        page: page.map(|page| page.parse().unwrap()),
        action: action.map(|action| action.parse().unwrap()),
        ..Window::default()
    }
}

#[test]
fn no_option_opens_home_as_always() {
    assert_eq!(opening(&Window::default()), Ok(Opening::default()));
}

#[test]
fn a_page_opens_by_its_id() {
    let opened = opening(&window(Some("maintenance"), None)).unwrap();

    assert_eq!(opened.page, Some(PageId::Maintenance));
    assert_eq!(opened.activation().page.unwrap().as_ref(), "maintenance");
}

#[test]
fn the_setup_assistant_flag_is_the_setup_page() {
    let opened = opening(&Window {
        setup_assistant: true,
        ..Window::default()
    })
    .unwrap();

    assert_eq!(opened.page, Some(PageId::Setup));
}

#[test]
fn a_report_runs_on_the_diagnostics_page() {
    let opened = opening(&window(Some("diagnostics"), Some("inxi"))).unwrap();

    assert_eq!(
        opened.action,
        Some(PageAction::Collect(Selection::One(Collector::Inxi)))
    );
    assert_eq!(opened.activation().action.unwrap().as_ref(), "inxi");
}

#[test]
fn an_unknown_page_or_action_is_refused_before_the_window_opens() {
    let page = opening(&window(Some("nowhere"), None))
        .unwrap_err()
        .to_string();

    assert!(page.contains("nowhere"), "{page}");

    let action = opening(&window(Some("maintenance"), Some("inxi")))
        .unwrap_err()
        .to_string();

    assert!(action.contains("inxi"), "{action}");
}

/// The second line is the hint, as `error_text` prints it.
#[test]
fn an_unknown_page_hints_at_the_list_on_a_line_of_its_own() {
    let page = opening(&window(Some("nowhere"), None))
        .unwrap_err()
        .to_string();

    assert_eq!(
        page,
        "`nowhere` is no page\n`garuda-toolbox pages` lists the page ids"
    );
}
