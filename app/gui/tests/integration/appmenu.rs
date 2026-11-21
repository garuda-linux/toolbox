use std::sync::Arc;

use egui_kittest::Harness;
use parking_lot::Mutex;
use toolbox_appmenu::{Entry, Item, Menu, Toggle};
use toolbox_gui::appmenu::{MenuBar, StartMenuBar};
use toolbox_gui::command::{CommandSender, UiCommand};
use toolbox_gui::page::PageId;
use toolbox_gui::queue::Queue;
use toolbox_gui::{App, Setup};

use crate::common::{installing, launch};

type Shown = Arc<Mutex<Vec<Menu<UiCommand>>>>;

/// The menus the app sent, and the bus it gave the menu for clicks.
struct Recorded {
    shown: Shown,
    clicks: Arc<Mutex<Option<CommandSender>>>,
}

struct Bar(Shown);

impl MenuBar for Bar {
    fn show(&mut self, menu: Menu<UiCommand>) {
        self.0.lock().push(menu);
    }
}

fn launched(queue: Queue) -> (Harness<'static, App>, Recorded) {
    let recorded = Recorded {
        shown: Shown::default(),
        clicks: Arc::default(),
    };

    let shown = Arc::clone(&recorded.shown);
    let clicks = Arc::clone(&recorded.clicks);
    let harness = launch(Setup {
        queue,
        menu_bar: Some(StartMenuBar::new(move |_, commands| {
            *clicks.lock() = Some(commands);
            let bar: Box<dyn MenuBar> = Box::new(Bar(shown));
            Some(bar)
        })),
        ..Setup::default()
    });

    (harness, recorded)
}

fn last(recorded: &Recorded) -> Menu<UiCommand> {
    recorded.shown.lock().last().cloned().unwrap()
}

fn item(menu: &Menu<UiCommand>, action: UiCommand) -> Item<UiCommand> {
    menu.menus
        .iter()
        .flat_map(|submenu| &submenu.entries)
        .find_map(|entry| match entry {
            Entry::Item(item) if item.action == action => Some(item.clone()),
            Entry::Item(_) | Entry::Separator => None,
        })
        .unwrap()
}

#[test]
fn the_menu_follows_the_queue_and_is_not_sent_again_unchanged() {
    let (mut empty, recorded) = launched(Queue::in_memory(installing(&[])));
    let (_queued, with_changes) = launched(Queue::in_memory(installing(&["firefox"])));

    assert!(!item(&last(&recorded), UiCommand::ReviewChanges).enabled);
    assert!(item(&last(&with_changes), UiCommand::ReviewChanges).enabled);
    let sent = recorded.shown.lock().len();
    empty.run();
    assert_eq!(recorded.shown.lock().len(), sent);
}

#[test]
fn a_click_in_the_menu_opens_the_page_and_the_menu_marks_it() {
    let (mut harness, recorded) = launched(Queue::in_memory(installing(&[])));
    let home = UiCommand::Navigate(PageId::Home);
    let packages = UiCommand::Navigate(PageId::Packages);
    assert_eq!(item(&last(&recorded), home).toggle, Toggle::Radio(true));

    recorded.clicks.lock().as_ref().unwrap().send(packages);
    harness.run();

    assert_eq!(item(&last(&recorded), packages).toggle, Toggle::Radio(true));
    assert_eq!(item(&last(&recorded), home).toggle, Toggle::Radio(false));
}
