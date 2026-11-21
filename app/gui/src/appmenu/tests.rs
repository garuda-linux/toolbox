use toolbox_appmenu::{Entry, Item, Menu, Modifier, Toggle};

use super::{MenuState, menu};
use crate::command::UiCommand;
use crate::page::PageId;
use crate::zoom::Zoom;

fn state() -> MenuState {
    MenuState {
        language: crate::i18n::FALLBACK,
        page: PageId::Home,
        listed: vec![
            PageId::Home,
            PageId::Setup,
            PageId::Packages,
            PageId::Settings,
        ],
        sidebar_expanded: true,
        console_open: false,
        queued: false,
        zoom: Zoom::default(),
    }
}

fn built(state: &MenuState) -> Menu<UiCommand> {
    menu(state, ToOwned::to_owned)
}

fn items(menu: &Menu<UiCommand>, title: &str) -> Vec<Item<UiCommand>> {
    menu.menus
        .iter()
        .find(|submenu| submenu.label == title)
        .unwrap()
        .entries
        .iter()
        .filter_map(|entry| match entry {
            Entry::Item(item) => Some(item.clone()),
            Entry::Separator => None,
        })
        .collect()
}

fn item(menu: &Menu<UiCommand>, title: &str, action: UiCommand) -> Item<UiCommand> {
    items(menu, title)
        .into_iter()
        .find(|item| item.action == action)
        .unwrap()
}

#[test]
fn the_bar_holds_file_view_go_and_help() {
    let titles: Vec<String> = built(&state())
        .menus
        .into_iter()
        .map(|submenu| submenu.label)
        .collect();

    assert_eq!(
        titles,
        ["appmenu-file", "appmenu-view", "appmenu-go", "appmenu-help"]
    );
}

#[test]
fn nothing_applies_the_queue_and_review_waits_for_a_change() {
    let empty = built(&state());
    let queued = built(&MenuState {
        queued: true,
        ..state()
    });

    let every = |menu: &Menu<UiCommand>| -> Vec<UiCommand> {
        ["appmenu-file", "appmenu-view", "appmenu-go", "appmenu-help"]
            .into_iter()
            .flat_map(|title| items(menu, title))
            .map(|item| item.action)
            .collect()
    };

    assert!(!every(&queued).contains(&UiCommand::Apply));
    assert!(!item(&empty, "appmenu-file", UiCommand::ReviewChanges).enabled);
    assert!(!item(&empty, "appmenu-file", UiCommand::ClearChanges).enabled);
    let review = item(&queued, "appmenu-file", UiCommand::ReviewChanges);
    assert!(review.enabled);
    assert_eq!(review.label, "appmenu-review-and-apply");
}

#[test]
fn go_lists_the_sidebar_pages_by_group_with_the_open_one_marked() {
    let menu = built(&MenuState {
        page: PageId::Packages,
        ..state()
    });

    let pages: Vec<(UiCommand, Toggle)> = items(&menu, "appmenu-go")
        .into_iter()
        .filter(|item| matches!(item.action, UiCommand::Navigate(_)))
        .map(|item| (item.action, item.toggle))
        .collect();

    assert_eq!(
        pages,
        [
            (UiCommand::Navigate(PageId::Home), Toggle::Radio(false)),
            (UiCommand::Navigate(PageId::Setup), Toggle::Radio(false)),
            (UiCommand::Navigate(PageId::Packages), Toggle::Radio(true)),
            (UiCommand::Navigate(PageId::Settings), Toggle::Radio(false)),
        ]
    );

    let go = &menu.menus[2].entries;
    let separators = go
        .iter()
        .filter(|entry| **entry == Entry::Separator)
        .count();

    assert_eq!(separators, 3, "one before each of the three groups");
}

#[test]
fn view_marks_sidebar_and_console_and_stops_zoom_at_its_bounds() {
    let menu = built(&MenuState {
        console_open: true,
        zoom: (0_u8..10).fold(Zoom::default(), |zoom, _| zoom.larger()),
        ..state()
    });

    assert_eq!(
        item(&menu, "appmenu-view", UiCommand::ToggleSidebar).toggle,
        Toggle::Check(true)
    );
    assert_eq!(
        item(&menu, "appmenu-view", UiCommand::ToggleConsole).toggle,
        Toggle::Check(true)
    );
    assert!(!item(&menu, "appmenu-view", UiCommand::ZoomIn).enabled);
    assert!(item(&menu, "appmenu-view", UiCommand::ZoomOut).enabled);
}

#[test]
fn items_show_the_shortcuts_the_window_answers() {
    let menu = built(&state());

    let quit = item(&menu, "appmenu-file", UiCommand::Quit)
        .shortcut
        .unwrap();
    assert_eq!((quit.modifiers, quit.key), (vec![Modifier::Control], "Q"));
    let zoom = item(&menu, "appmenu-view", UiCommand::ZoomIn)
        .shortcut
        .unwrap();
    assert_eq!(zoom.key, "plus");
    let help = item(&menu, "appmenu-help", UiCommand::Help)
        .shortcut
        .unwrap();
    assert_eq!((help.modifiers, help.key), (Vec::new(), "F1"));
    let home = item(&menu, "appmenu-go", UiCommand::Navigate(PageId::Home))
        .shortcut
        .unwrap();
    assert_eq!(home.key, "1");
    assert_eq!(
        item(&menu, "appmenu-file", UiCommand::ReviewChanges).shortcut,
        None
    );
}
