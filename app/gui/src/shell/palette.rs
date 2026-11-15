//! The command palette (Ctrl+K): every page and command by name, filtered
//! as the user types, then the Discover apps whose title holds the text,
//! with the Maintenance page's operations, tools and repairs before them,
//! each group under its label. The typing keeps the focus, Up and Down
//! move the marked row, Enter takes it.

mod found;
mod row;
#[cfg(test)]
mod tests;

use std::rc::Rc;

use egui::{Context, Id, Key, Modifiers, ScrollArea};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    DialogWidth, ModalWrapper, SearchTexts, menu_heading, search_field,
};

use self::found::{Asked, Row};
use crate::command::{Command, CommandSender};
use crate::i18n;
use crate::pages::PageContext;

/// Whether the palette is open and what was typed. Not persisted.
#[derive(Debug, Default)]
pub struct Palette {
    open: bool,
    query: String,
    /// The row Enter takes, back to the first as the text changes.
    marked: usize,
    /// The rows of the last frame, while their inputs hold.
    found: Option<(Asked, Rc<[Row]>)>,
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

        let rows = found::cached(&mut self.found, ctx, context, &self.query);

        let title = i18n::text(ctx, "command-open-palette");
        let hint = i18n::text(ctx, "palette-search");
        let clear = i18n::text(ctx, "search-clear");

        // From the top, so the field stays put as the list changes length.
        let modal = ModalWrapper::new(Id::unique("toolbox_gui::palette"), &title)
            .instant()
            .top_anchored()
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
                let search = search_field(ui, texts.name, &mut self.query, texts, width);
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

                let (clicked, pointed) = ScrollArea::vertical()
                    .max_height(ui.tokens().sizes.palette_list_height)
                    .show(ui, |ui| self.list(ui, &rows, moved, width))
                    .inner;

                if let Some(index) = pointed {
                    self.marked = index;
                }

                entered.or(clicked)
            });

        if modal.closed {
            self.open = false;
        }

        if let Some((command, by_key)) = modal.inner {
            send(commands, command, by_key);
            self.open = false;
        }

        // A closed palette need not keep the snapshot its rows were found in.
        if !self.open {
            self.found = None;
        }
    }

    /// The rows under their group labels, the marked one shown as selected
    /// and scrolled to after a key `moved` it. Returns a clicked row's
    /// command and whether the click came from the keyboard, and the row
    /// the pointer moved onto.
    ///
    /// Only a moving pointer takes the mark, so a list scrolled by the
    /// keys under a resting one keeps the key's mark.
    fn list(
        &self,
        ui: &mut egui::Ui,
        rows: &[Row],
        moved: bool,
        width: f32,
    ) -> (Option<(Command, bool)>, Option<usize>) {
        let pointer_moved = ui.input(|input| {
            input
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::PointerMoved(_)))
        });

        let mut chosen = None;
        let mut pointed = None;
        let mut section = None;
        for (index, row) in rows.iter().enumerate() {
            if section != Some(row.section) {
                section = Some(row.section);
                menu_heading(ui, &row.section.label(ui.ctx()));
            }

            let marked = index == self.marked;
            let response = row::show(ui, row, marked, width);
            if marked && moved {
                response.scroll_to_me(None);
            }

            if pointer_moved && response.hovered() {
                pointed = Some(index);
            }

            if response.clicked() {
                let by_key = !response.clicked_by(egui::PointerButton::Primary);
                chosen = Some((row.command.clone(), by_key));
            }
        }

        (chosen, pointed)
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
