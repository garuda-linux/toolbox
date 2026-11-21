//! The window's global menu: what it holds for the app's state, and the
//! port that shows it. Plasma shows it in the title bar or a panel. Where
//! no desktop asks for it, nothing is shown.
//!
//! Any process of the user can click it, so it holds nothing that applies
//! the queue: File opens the review, where Apply is.

#[cfg(test)]
mod tests;

use std::fmt;

use eframe::CreationContext;
use egui::{Key, KeyboardShortcut};
use fluent_templates::LanguageIdentifier;
use tokio::runtime::Handle;
use tokio::sync::watch;
use toolbox_appmenu::{Entry, Item, Menu, Modifier, Shortcut, Started, Submenu, Toggle};
use toolbox_instance::bus::BUS_LIMIT;

use crate::command::{CommandSender, UiCommand};
use crate::page::PageId;
use crate::zoom::Zoom;

/// What the menu shows. A frame sends a new menu only when this changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuState {
    pub language: LanguageIdentifier,
    pub page: PageId,
    /// The pages the sidebar lists, in its order.
    pub listed: Vec<PageId>,
    pub sidebar_expanded: bool,
    pub console_open: bool,
    pub queued: bool,
    pub zoom: Zoom,
}

/// Where the menu goes: the session bus, or a test's list.
pub trait MenuBar {
    fn show(&mut self, menu: Menu<UiCommand>);
}

/// Starts the menu of the window `App::new` makes, with the bus clicks
/// are sent to. `None` where the desktop has no global menu.
pub struct StartMenuBar(Box<Start>);

type Start = dyn FnOnce(&CreationContext<'_>, CommandSender) -> Option<Box<dyn MenuBar>>;

/// Looks a text up by its Fluent id.
type Text<'text> = &'text dyn Fn(&str) -> String;

/// The menu bar and what it shows now.
pub struct Shown {
    bar: Box<dyn MenuBar>,
    state: Option<MenuState>,
}

/// The menu served on the session bus. A watch keeps only the newest menu
/// for the one task that sends them.
struct Served {
    menus: watch::Sender<Menu<UiCommand>>,
}

impl StartMenuBar {
    pub fn new(
        start: impl FnOnce(&CreationContext<'_>, CommandSender) -> Option<Box<dyn MenuBar>> + 'static,
    ) -> Self {
        Self(Box::new(start))
    }
}

impl fmt::Debug for StartMenuBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("StartMenuBar")
    }
}

impl Shown {
    pub fn start(
        start: Option<StartMenuBar>,
        cc: &CreationContext<'_>,
        commands: &CommandSender,
    ) -> Option<Self> {
        let bar = (start?.0)(cc, commands.clone())?;
        Some(Self { bar, state: None })
    }

    /// Sends the menu for `state` when it differs from the one shown.
    pub fn follow(&mut self, state: MenuState, text: impl Fn(&str) -> String) {
        if self.state.as_ref() != Some(&state) {
            self.bar.show(menu(&state, text));
            self.state = Some(state);
        }
    }
}

impl MenuBar for Served {
    fn show(&mut self, menu: Menu<UiCommand>) {
        self.menus.send_replace(menu);
    }
}

/// The menu on the session bus, tied to the window. `App::new` waits for
/// it at most [`BUS_LIMIT`], as the window's handles hold only during that
/// call.
pub fn on_session_bus(runtime: Handle) -> StartMenuBar {
    StartMenuBar::new(move |cc, commands| {
        let ctx = cc.egui_ctx.clone();
        let clicks = move |command: UiCommand| {
            commands.send(command);
            ctx.request_repaint();
        };

        let empty = Menu { menus: Vec::new() };
        let started = runtime.block_on(async {
            tokio::time::timeout(BUS_LIMIT, toolbox_appmenu::start(cc, &empty, clicks)).await
        });

        let exported = match started {
            Ok(Ok(Started { exported, bound })) => {
                if !bound {
                    tracing::debug!("the desktop shows no global menu, it is only served");
                }

                #[cfg(feature = "selftest")]
                if let Some(name) = exported.connection().unique_name() {
                    crate::selftest::menu::announce(&cc.egui_ctx, name.to_string());
                }

                exported
            }
            Ok(Err(error)) => {
                tracing::warn!("the global menu is off: {error}");
                return None;
            }
            Err(_) => {
                tracing::warn!("the global menu is off: the session bus took over {BUS_LIMIT:?}");
                return None;
            }
        };

        let (menus, mut fresh) = watch::channel(empty);
        runtime.spawn(async move {
            while fresh.changed().await.is_ok() {
                let menu = fresh.borrow_and_update().clone();
                if let Err(error) = exported.update(&menu).await {
                    tracing::warn!("the global menu missed a change: {error}");
                }
            }
        });

        let bar: Box<dyn MenuBar> = Box::new(Served { menus });
        Some(bar)
    })
}

pub fn menu(state: &MenuState, lookup: impl Fn(&str) -> String) -> Menu<UiCommand> {
    let text: Text<'_> = &lookup;
    let submenu = |id: &str, entries| Submenu {
        label: text(id),
        entries,
    };

    Menu {
        menus: vec![
            submenu("appmenu-file", file(state, text)),
            submenu("appmenu-view", view(state, text)),
            submenu("appmenu-go", go(state, text)),
            submenu("appmenu-help", vec![plain(text, UiCommand::Help)]),
        ],
    }
}

fn file(state: &MenuState, text: Text<'_>) -> Vec<Entry<UiCommand>> {
    vec![
        enabled(
            labelled(text, UiCommand::ReviewChanges, "appmenu-review-and-apply"),
            state.queued,
        ),
        enabled(
            labelled(text, UiCommand::ClearChanges, "appmenu-clear-changes"),
            state.queued,
        ),
        Entry::Separator,
        plain(text, UiCommand::Quit),
    ]
}

fn view(state: &MenuState, text: Text<'_>) -> Vec<Entry<UiCommand>> {
    vec![
        toggled(
            labelled(text, UiCommand::ToggleSidebar, "appmenu-sidebar-expanded"),
            Toggle::Check(state.sidebar_expanded),
        ),
        toggled(
            labelled(text, UiCommand::ToggleConsole, "console-title"),
            Toggle::Check(state.console_open),
        ),
        Entry::Separator,
        enabled(
            labelled(text, UiCommand::ZoomIn, UiCommand::ZoomIn.text_id()),
            state.zoom.larger() != state.zoom,
        ),
        enabled(
            labelled(text, UiCommand::ZoomOut, UiCommand::ZoomOut.text_id()),
            state.zoom.smaller() != state.zoom,
        ),
        plain(text, UiCommand::ZoomReset),
    ]
}

/// Search and the palette, then the sidebar's pages, its groups parted by
/// lines.
fn go(state: &MenuState, text: Text<'_>) -> Vec<Entry<UiCommand>> {
    let mut go = vec![
        plain(text, UiCommand::FocusSearch),
        plain(text, UiCommand::OpenPalette),
    ];

    let mut group = None;
    for page in &state.listed {
        if group != Some(page.group()) {
            group = Some(page.group());
            go.push(Entry::Separator);
        }

        let command = UiCommand::Navigate(*page);
        go.push(toggled(
            labelled(text, command, page.text_id()),
            Toggle::Radio(*page == state.page),
        ));
    }

    go
}

fn labelled(text: Text<'_>, command: UiCommand, id: &str) -> Item<UiCommand> {
    let mut item = Item::new(command, text(id));
    item.shortcut = command.shortcut().map(shortcut);
    item
}

/// The command under the name the palette gives it.
fn plain(text: Text<'_>, command: UiCommand) -> Entry<UiCommand> {
    Entry::Item(labelled(text, command, command.text_id()))
}

const fn enabled(mut item: Item<UiCommand>, enabled: bool) -> Entry<UiCommand> {
    item.enabled = enabled;
    Entry::Item(item)
}

const fn toggled(mut item: Item<UiCommand>, toggle: Toggle) -> Entry<UiCommand> {
    item.toggle = toggle;
    Entry::Item(item)
}

/// egui's Command is Ctrl on Linux. Qt names plus and minus by word.
fn shortcut(keys: KeyboardShortcut) -> Shortcut {
    let held = [
        (
            keys.modifiers.command || keys.modifiers.ctrl,
            Modifier::Control,
        ),
        (keys.modifiers.alt, Modifier::Alt),
        (keys.modifiers.shift, Modifier::Shift),
    ];

    let key = if keys.logical_key == Key::Plus {
        "plus"
    } else if keys.logical_key == Key::Minus {
        "minus"
    } else {
        keys.logical_key.name()
    };

    Shortcut {
        modifiers: held
            .into_iter()
            .filter_map(|(down, modifier)| down.then_some(modifier))
            .collect(),
        key,
    }
}
