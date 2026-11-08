//! The command palette (Ctrl+K): every page and command by name, filtered
//! as the user types, then the Discover apps whose title holds the text,
//! with the Maintenance page's operations, tools and repairs before them.
//! The typing keeps the focus, Up and Down move the marked row, Enter
//! takes it.

use egui::{Button, Context, Id, Key, Modifiers, RichText, ScrollArea, vec2};
use toolbox_core::features::search::Needle;
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{DialogWidth, ModalWrapper, SearchTexts, search_field};

use crate::command::{Command, CommandSender, DiscoverCommand, MaintenanceCommand, UiCommand};
use crate::page::PageId;
use crate::pages::discover::category_title;
use crate::pages::{PageContext, maintenance};
use crate::{i18n, page_ui};

/// The list's height before it scrolls, in points.
const LIST_HEIGHT: f32 = 380.0;
/// At most this many rows: a short text matches hundreds of apps.
const LIMIT: usize = 50;

/// Whether the palette is open and what was typed. Not persisted.
#[derive(Debug, Default)]
pub struct Palette {
    open: bool,
    query: String,
    /// The row Enter takes, back to the first as the text changes.
    marked: usize,
}

struct Row {
    name: String,
    detail: String,
    command: Command,
}

impl Palette {
    pub fn open(&mut self) {
        self.open = true;
        self.query.clear();
        self.marked = 0;
    }

    /// A command chosen from the keyboard goes out as
    /// [`Command::Keyed`], so what it opens shows at once.
    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>, commands: &CommandSender) {
        if !self.open {
            return;
        }

        let rows = self.matches(ctx, context);

        let title = i18n::text(ctx, "command-open-palette");
        let hint = i18n::text(ctx, "palette-search");
        let clear = i18n::text(ctx, "search-clear");
        let modal = ModalWrapper::new(Id::unique("toolbox_gui::palette"), &title)
            .instant()
            .width(DialogWidth::Small)
            .show(ctx, |ui| {
                let width = ui.available_width();
                let texts = SearchTexts {
                    name: &title,
                    hint: &hint,
                    clear: &clear,
                };

                // Taken before the field sees them, which would move its
                // cursor with them.
                let moved = self.move_mark(ui, rows.len());
                let search = search_field(ui, &mut self.query, texts, width);
                if search.changed() {
                    self.marked = 0;
                }

                ui.add_space(ui.tokens().sizes.control_gap);

                search.request_focus();

                let entered = ui.input(|input| input.key_pressed(Key::Enter));
                let entered = entered
                    .then(|| rows.get(self.marked))
                    .flatten()
                    .map(|row| (row.command.clone(), true));

                let clicked = ScrollArea::vertical()
                    .max_height(LIST_HEIGHT)
                    .show(ui, |ui| self.list(ui, &rows, moved, width))
                    .inner;

                entered.or(clicked)
            });

        if modal.closed {
            self.open = false;
        }

        if let Some((command, by_key)) = modal.inner {
            send(commands, command, by_key);
            self.open = false;
        }
    }

    /// The rows, the marked one shown as selected and scrolled to after a
    /// key `moved` it. Returns a clicked row's command and whether the
    /// click came from the keyboard.
    fn list(
        &self,
        ui: &mut egui::Ui,
        rows: &[Row],
        moved: bool,
        width: f32,
    ) -> Option<(Command, bool)> {
        let mut chosen = None;
        for (index, row) in rows.iter().enumerate() {
            // The marked row is what Enter takes, and its shortcut takes
            // the colour of selected text.
            let marked = index == self.marked;
            let colour = if marked {
                ui.visuals().selection.stroke.color
            } else {
                ui.visuals().weak_text_color()
            };

            let mut button =
                Button::selectable(marked, row.name.as_str()).min_size(vec2(width, 0.0));

            // egui joins the name and an empty shortcut with a space,
            // which a screen reader then reads out.
            if !row.detail.is_empty() {
                button = button.shortcut_text(RichText::new(&row.detail).color(colour));
            }

            let response = ui.add(button);
            if marked && moved {
                response.scroll_to_me(None);
            }

            if response.clicked() {
                let by_key = !response.clicked_by(egui::PointerButton::Primary);
                chosen = Some((row.command.clone(), by_key));
            }
        }

        chosen
    }

    /// Moves the mark by the Up and Down pressed this frame, wrapping at
    /// the ends. Returns whether it moved.
    fn move_mark(&mut self, ui: &egui::Ui, count: usize) -> bool {
        let (up, down) = ui.input_mut(|input| {
            let up = input.count_and_consume_key(Modifiers::NONE, Key::ArrowUp);
            let down = input.count_and_consume_key(Modifiers::NONE, Key::ArrowDown);

            (up, down)
        });

        if count == 0 || up == down {
            return false;
        }

        let marked = self.marked.min(count - 1);
        let steps = (down + count - up % count) % count;
        self.marked = (marked + steps) % count;
        true
    }

    /// Every page and command whose name holds the typed text, then the
    /// apps this system offers whose title holds it. Apps only show for a
    /// typed text, there are too many to list.
    fn matches(&self, ctx: &Context, context: &PageContext<'_>) -> Vec<Row> {
        let needle = Needle::new(&self.query);
        let pages = PageId::ALL
            .into_iter()
            .filter(|page| page_ui::listed(ctx, *page))
            .map(UiCommand::Navigate);

        // Reviewing has nothing to show while the queue is empty.
        let review = (!context.changes.is_empty()).then_some(UiCommand::ReviewChanges);
        let commands = pages
            .chain(UiCommand::ACTIONS)
            .chain(review)
            .filter(|command| *command != UiCommand::OpenPalette)
            .map(|command| Row {
                name: i18n::text(ctx, command.text_id()).into_owned(),
                detail: command
                    .shortcut()
                    .map(|shortcut| ctx.format_shortcut(&shortcut))
                    .unwrap_or_default(),
                command: command.into(),
            })
            .filter(|row| needle.is_in(&row.name));

        let apps = (!needle.is_empty())
            .then(|| {
                context
                    .catalogs
                    .apps
                    .apps
                    .offered(context.snapshot, context.changes)
            })
            .into_iter()
            .flatten()
            .filter(|(_, hit)| hit.title_matches(&needle))
            .map(|(_, hit)| {
                let title = hit.entry.title();
                let category = category_title(ctx, hit.category.id(), hit.category.title());
                Row {
                    name: title.to_owned(),
                    detail: format!("{} · {category}", i18n::text(ctx, "discover-palette-app")),
                    command: DiscoverCommand::Reveal(title.to_owned()).into(),
                }
            });

        let page = i18n::text(ctx, PageId::Maintenance.text_id());
        let maintenance = (!needle.is_empty())
            .then(maintenance::palette::entries)
            .into_iter()
            .flatten()
            .map(|(id, tab)| Row {
                name: i18n::text(ctx, &id).into_owned(),
                detail: format!("{page} · {}", i18n::text(ctx, tab.text_id())),
                command: MaintenanceCommand::Reveal(tab).into(),
            })
            .filter(|row| needle.is_in(&row.name));

        commands
            .chain(maintenance)
            .chain(apps)
            .take(LIMIT)
            .collect()
    }
}

/// Sends what the palette chose. From the keyboard a page or shell
/// command goes as [`Command::Keyed`], as its shortcut would.
fn send(commands: &CommandSender, command: Command, by_key: bool) {
    commands.send(match command {
        Command::Ui(command) if by_key => Command::Keyed(command),
        other @ (Command::Ui(_) | Command::App(_) | Command::Confirmed(_) | Command::Keyed(_)) => {
            other
        }
    });
}
