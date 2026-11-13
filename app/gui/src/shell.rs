mod changes;
mod console_panel;
mod palette;
mod sidebar;
mod status_bar;

pub use changes::ChangesState;
pub use console_panel::focus::on_open as focus_console_on_open;
#[cfg(any(test, feature = "testing"))]
pub use console_panel::layer as console_layer;
use egui::{CentralPanel, Frame, Panel, Rect, Ui};
pub use palette::Palette;
#[cfg(any(test, feature = "testing"))]
pub use sidebar::COLLAPSED_WIDTH;
use toolbox_core::ChangeView;
use toolbox_gui_kit::fonts::BODY;
use toolbox_gui_kit::toasts::{ToastTexts, Toasts};
use toolbox_gui_kit::widgets::{
    PageParts, cover_page_bottom, page_layout_shaped, start_page_entry,
};
use toolbox_gui_kit::{UiExt as _, tokens};
use toolbox_plan::protocol::Mode;

use crate::command::CommandSender;
use crate::console::Console;
use crate::i18n;
use crate::page::PageId;
use crate::page_ui::GroupLook as _;
use crate::pages::{PageContext, Pages};
use crate::snapshot::Reading;

/// A pane's title size, the sidebar's and the Changes popover's alike.
const PANE_TITLE: f32 = BODY * 1.07;

/// What the shell draws from. The app owns it, the shell only reads it,
/// except for state that belongs to the window alone: dragging the
/// sidebar shut, the palette's text, an open question, and the Console's
/// terminal (its scroll position and size).
pub struct View<'app> {
    pub page: PageId,
    pub pages: &'app mut Pages,
    pub context: &'app PageContext<'app>,
    pub dry_run: bool,
    pub sidebar_expanded: &'app mut bool,
    pub console_open: bool,
    pub snap: Snap,
    pub console: &'app mut Console,
    pub palette: &'app mut Palette,
    pub changes: &'app [ChangeView],
    pub changes_ui: &'app mut ChangesState,
    pub reading: &'app Reading,
    /// Why Apply waits, as the id of its text: a restored queue the
    /// pages' guards have not checked yet.
    pub held: Option<&'static str>,
    pub toasts: &'app mut Toasts,
}

/// What a shortcut changed this frame, so it jumps there rather than
/// slides: motion follows the pointer, never the keyboard.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Snap {
    pub sidebar: bool,
    pub console: bool,
    /// A shortcut opened another page, so it shows in place at once.
    pub page: bool,
}

pub fn show(ui: &mut Ui, view: View<'_>, commands: &CommandSender) {
    let View {
        page,
        pages,
        context,
        dry_run,
        sidebar_expanded,
        console_open,
        snap,
        console,
        palette,
        changes,
        changes_ui,
        reading,
        held,
        toasts,
    } = view;

    let status = status_bar::show(ui, reading, Mode::of_dry_run(dry_run));
    sidebar::show(ui, page, sidebar_expanded, snap.sidebar, commands);

    let changes = shown_changes(pages, page, changes_ui.open, changes);
    let queued = changes::Queued {
        changes,
        held,
        running: console.running(),
        console_open,
    };

    let sheet = console_panel::Sheet {
        open: console_open,
        instant: snap.console,
        dry_run,
    };

    // The sheet lies over the page, so the page keeps its size. While it
    // shows, the queue sits in its header and the band gives the page
    // its room back.
    let out = console_panel::out(ui.ctx(), sheet);
    let band = (out <= 0.0 && !changes.is_empty()).then(|| changes_band(ui));
    let page_rect = page_area(ui, page, snap.page, pages, context);

    let at = console_panel::At {
        sheet,
        place: console_panel::Place {
            room: page_rect,
            out,
        },
    };

    let sheet_shown = console_panel::show(ui.ctx(), at, console, commands, |ui| {
        changes::inline(ui, queued, changes_ui, commands);
    });

    if !sheet_shown {
        let room = band.unwrap_or(page_rect);
        changes::floating(ui.ctx(), room, queued, changes_ui, commands);
    }

    status
        .inspect(|rect| status_bar::fill(ui, *rect, reading, Mode::of_dry_run(dry_run), commands));
    let ctx = ui.ctx().clone();
    let texts = ToastTexts {
        name: &i18n::text(&ctx, "toast-name"),
        close: &i18n::text(&ctx, "dialog-close"),
    };

    status_bar::floating(&ctx, page_rect, reading, Mode::of_dry_run(dry_run));
    toasts.show(&ctx, page_rect, texts);
    palette.show(&ctx, context, commands);
}

/// A page with its own way to apply shows the queue as empty.
fn shown_changes<'list>(
    pages: &mut Pages,
    page: PageId,
    open: bool,
    changes: &'list [ChangeView],
) -> &'list [ChangeView] {
    if pages.view(page).queue_shown().shows(open) {
        changes
    } else {
        &[]
    }
}

fn changes_band(ui: &mut Ui) -> Rect {
    let tokens = tokens::current(ui.ctx());
    Panel::bottom("changes_band")
        .exact_size(tokens.sizes.changes_band)
        .resizable(false)
        .show_separator_line(false)
        .frame(Frame::new().fill(tokens.window))
        .show(ui, |_ui| {})
        .response
        .rect
}

/// The page in the room the bars left, `jump` when a shortcut opened it.
/// Returns where it lies.
fn page_area(
    ui: &mut Ui,
    page: PageId,
    jump: bool,
    pages: &mut Pages,
    context: &PageContext<'_>,
) -> Rect {
    open_page(ui.ctx(), page, jump);
    let window = tokens::current(ui.ctx()).window;
    CentralPanel::default()
        .frame(Frame::new().fill(window))
        .show(ui, |ui| {
            // The page paints on under the Console, so the sheet's frost
            // shows it blurred. Its scroll area ends at the sheet's top.
            if let Some(cover) = console_panel::cover(ui.ctx()) {
                cover_page_bottom(ui.ctx(), ui.max_rect().bottom() - cover.top());
            }

            ui.name_pane(&i18n::text(ui.ctx(), "pane-page"));
            let parts = PageParts {
                header: |shown: &mut Pages, area: &mut Ui| shown.view(page).header(area, context),
                body: |shown: &mut Pages, area: &mut Ui| shown.view(page).ui(area, context),
                footer: |shown: &mut Pages, area: &mut Ui| shown.view(page).footer(area, context),
            };

            let place = pages.view(page).place();
            let shape = pages.view(page).shape();
            page_layout_shaped(ui, (page, place), pages, parts, shape);
        })
        .response
        .rect
}

/// Gives the page's tiles its group's hue. Another page the pointer opened
/// fades in. A shortcut's page and another tab of the same page show in
/// place.
fn open_page(ctx: &egui::Context, page: PageId, jump: bool) {
    tokens::set_page_hue(ctx, page.group().hue());

    let shown = egui::Id::unique("toolbox_gui::shell::shown_page");
    let before = ctx.data_mut(|data| {
        let before = data.get_temp::<&'static str>(shown);
        data.insert_temp(shown, page.slug());
        before
    });

    if before.is_some_and(|before| before != page.slug()) && !jump {
        start_page_entry(ctx);
    }
}
