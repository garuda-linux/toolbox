mod changes;

pub use self::changes::{BuildNote, Hold};
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
use toolbox_core::ChangeView;
use toolbox_gui_kit::fonts::BODY;
use toolbox_gui_kit::toasts::{ToastTexts, Toasts};
use toolbox_gui_kit::widgets::{
    PageParts, cover_page_bottom, page_layout_shaped, push_height, start_page_entry,
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
const PANE_TITLE: f32 = BODY;

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
    /// Quit was confirmed during a run, the window closes once it stopped.
    pub stopping: bool,
    /// Why Apply waits: a restored queue the pages' guards have not checked
    /// yet, or the build orders of queued AUR packages.
    pub held: Option<&'app Hold>,
    /// The notes of the queued AUR builds, by package name.
    pub build_notes: &'app [(String, BuildNote)],
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

#[expect(
    clippy::too_many_lines,
    reason = "the shell draws its parts in one order, and each part is a line or two of the view"
)]
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
        stopping,
        held,
        build_notes,
        toasts,
    } = view;

    let status = status_bar::show(ui, reading, Mode::of_dry_run(dry_run));
    let side_reading = status_bar::reads_apart(reading, Mode::of_dry_run(dry_run));
    sidebar::show(
        ui,
        (page, side_reading),
        sidebar_expanded,
        snap.sidebar,
        commands,
    );

    let changes = shown_changes(pages, page, changes_ui.open, changes);
    let queued = changes::Queued {
        changes,
        held,
        notes: build_notes,
        running: console.running(),
        console_open,
    };

    let sheet = console_sheet(console_open, snap, dry_run);

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
        editing: editing(context),
    };

    let sheet_shown = console_panel::show(ui.ctx(), at, console, commands, |ui| {
        changes::inline(ui, queued, changes_ui, commands);
    });

    if sheet_shown.is_none() {
        let room = band.unwrap_or(page_rect);
        changes::footer(ui.ctx(), room, queued, changes_ui, commands);
    }

    status
        .inspect(|rect| status_bar::fill(ui, *rect, reading, Mode::of_dry_run(dry_run), commands));
    let ctx = ui.ctx().clone();

    // Notes stand above the Console's sheet, not over its last line.
    let notes_room = sheet_shown.map_or(page_rect, |shown| {
        Rect::from_min_max(page_rect.min, egui::pos2(page_rect.right(), shown.top()))
    });

    notices(&ctx, notes_room, stopping, toasts);
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

const fn console_sheet(open: bool, snap: Snap, dry_run: bool) -> console_panel::Sheet {
    console_panel::Sheet {
        open,
        instant: snap.console,
        dry_run,
    }
}

fn editing(context: &PageContext<'_>) -> bool {
    context
        .service::<crate::configs::Configs>()
        .session()
        .phase()
        .is_editing()
}

fn changes_band(ui: &mut Ui) -> Rect {
    let tokens = tokens::current(ui.ctx());
    Panel::bottom("changes_band")
        .exact_size(tokens.sizes.changes_chrome + push_height(ui))
        .resizable(false)
        .show_separator_line(false)
        .frame(Frame::new().fill(tokens.band))
        .show(ui, |_ui| {})
        .response
        .rect
}

/// The page in the room the bars left, `jump` when a shortcut opened it.
fn page_area(
    ui: &mut Ui,
    page: PageId,
    jump: bool,
    pages: &mut Pages,
    context: &PageContext<'_>,
) -> Rect {
    open_page(ui.ctx(), page, jump);
    let band = tokens::current(ui.ctx()).band;
    CentralPanel::default()
        .frame(Frame::new().fill(band))
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

fn notices(ctx: &egui::Context, page: Rect, stopping: bool, toasts: &mut Toasts) {
    let texts = ToastTexts {
        name: &i18n::text(ctx, "toast-name"),
    };

    if stopping {
        status_bar::note(ctx, page, &i18n::text(ctx, "close-stopping-title"));
    }

    toasts.show(ctx, page, texts);
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
