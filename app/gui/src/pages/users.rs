//! The Users and groups page: one tab of accounts, one of groups, and the
//! dialogs that change them.
//!
//! Most changes go into the queue. A new account and a new password run
//! at once, and the page shows how they went. The lock flags come from
//! the helper once the page opens.

mod dialogs;
mod groups;
mod list;
mod row;
#[cfg(test)]
mod tests;
mod texts;

use egui::Ui;
use toolbox_core::features::users::run::{PageRun, Retry, RetryStep};
use toolbox_core::features::users::view::{self, Shown};
use toolbox_gui_kit::widgets::{Tone, banner, card, empty_state, page_toolbar, skeleton_rows};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_services::locks::Locks;

use self::dialogs::{AddUser, Dialog, Next};
use super::header::{TabText, TabbedIds, search, tabbed_header_reading};
use super::rows::refused_banner;
use super::{PageContext, PageView, plan_running};
use crate::command::{AppCommand, UsersCommand};
use crate::i18n;
use crate::page::PageId;
use crate::users::Users;

/// Placeholder rows while the accounts are read, about as many as a
/// computer of one person has.
const SKELETON_ROWS: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Tab {
    #[default]
    Users,
    Groups,
}

impl TabText for Tab {
    fn text_id(self) -> &'static str {
        match self {
            Self::Users => "users-tab-users",
            Self::Groups => "users-tab-groups",
        }
    }
}

#[derive(Debug)]
pub struct UsersPage {
    tab: Tab,
    shown: Shown,
    dialog: Option<Dialog>,
    /// The add-user form while its run goes, without its passwords.
    retry: Option<Retry<Box<AddUser>>>,
    users: list::Rows,
    groups: groups::Rows,
}

impl Default for UsersPage {
    fn default() -> Self {
        Self {
            tab: Tab::default(),
            shown: Shown::people(),
            dialog: None,
            retry: None,
            users: list::Rows::new(),
            groups: groups::Rows::new(),
        }
    }
}

impl PageView for UsersPage {
    /// A dialog may hold a password, which must not wait for the return.
    /// A form kept for a retry would open on an outcome seen too late.
    fn left(&mut self, ctx: &egui::Context) {
        if let Some(mut dialog) = self.dialog.take() {
            dialog.forget(ctx);
        }

        self.retry = None;
    }

    fn place(&self) -> u64 {
        super::place_of(&self.tab)
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        let ids = TabbedIds {
            title: "page-users",
            subtitle: "page-users-subtitle",
            tabs: "users-tabs",
            refresh: "users-refresh",
        };

        let reading = context.service::<Users>().reading();
        let tabs = [Tab::Users, Tab::Groups];
        let done = tabbed_header_reading(ui, ids, &tabs, self.tab, reading);
        if let Some(picked) = done.picked {
            self.tab = picked;
        }

        if done.refresh {
            context.commands.send(UsersCommand::Refresh);
        }

        page_toolbar(ui, self, Self::search, |page, ui| match page.tab {
            Tab::Users => list::sort(ui, &mut page.shown),
            Tab::Groups => groups::sort(ui, &mut page.shown),
        });

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("users ui").entered();
        notes(ui, context, self.tab);
        plan_running(ui, context);
        match self.tab {
            Tab::Users => list::show(
                ui,
                context,
                (&mut self.shown, &mut self.users),
                &mut self.dialog,
            ),
            Tab::Groups => groups::show(
                ui,
                context,
                (&mut self.shown, &mut self.groups),
                &mut self.dialog,
            ),
        }

        let ctx = ui.ctx().clone();
        if let Some(dialog) = &mut self.dialog
            && dialog.show(&ctx, context) == Next::Close
        {
            if let Dialog::AddUser(form) = dialog {
                self.retry = form.retry();
            }

            dialog.forget(&ctx);
            self.dialog = None;
        }

        self.give_back(&ctx, context);
    }
}

impl UsersPage {
    fn search(&mut self, ui: &mut Ui) {
        let (field, (name, hint)) = match self.tab {
            Tab::Users => (
                &mut self.shown.user_search,
                ("users-search", "users-search-hint"),
            ),
            Tab::Groups => (
                &mut self.shown.group_search,
                ("groups-search", "groups-search-hint"),
            ),
        };

        search(ui, field, name, hint);
    }

    /// Opens the add-user dialog again once its run failed or was refused.
    /// Another dialog the user opened meanwhile stays, the form goes.
    fn give_back(&mut self, ctx: &egui::Context, context: &PageContext<'_>) {
        let Some(retry) = self.retry.take() else {
            return;
        };

        match retry.step(context.service::<Users>().run()) {
            RetryStep::Wait(waiting) => self.retry = Some(waiting),
            RetryStep::Reopen(form) => {
                if self.dialog.is_none() {
                    self.dialog = Some(Dialog::AddUser(form));

                    // It shows from the next frame, which nothing else asks for.
                    ctx.request_repaint();
                }
            }
            RetryStep::Forget => {}
        }
    }
}

/// A detail line of a row: two lines of it, cut as a catalog row cuts its
/// description, with the whole text in the tooltip.
pub fn detail_line(ui: &mut Ui, text: &str) {
    let font = toolbox_gui_kit::fonts::font(
        ui.style(),
        toolbox_gui_kit::fonts::Weight::Regular,
        toolbox_gui_kit::fonts::SMALL,
    );

    let secondary = toolbox_gui_kit::tokens::current(ui.ctx()).secondary;
    let job = toolbox_gui_kit::widgets::highlighted(ui, text, "", font, secondary);

    ui.add(egui::Label::new(crate::pages::toggles::control::cut(job)).wrap())
        .on_hover_text(text);
}

/// Whether the accounts were read, else placeholder rows of `row_height`
/// while they are read or the empty state that says why they were not.
pub fn accounts_ready(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (reading_id, unavailable_id): (&str, &str),
    row_height: f32,
) -> bool {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    if !context.probed {
        let name = text(reading_id);
        card(ui, |ui| skeleton_rows(ui, SKELETON_ROWS, row_height, &name));
        return false;
    }

    if !view::accounts_known(context.snapshot) {
        let detail = text("users-unavailable-detail");
        empty_state(ui, icons::HELP_ABOUT, &text(unavailable_id), &detail);
        return false;
    }

    true
}

fn notes(ui: &mut Ui, context: &PageContext<'_>, tab: Tab) {
    let ctx = ui.ctx().clone();
    let dismiss = i18n::text(&ctx, "users-dismiss");
    let refused = context.service::<Users>().refusal();
    if refused_banner(ui, refused, Tone::Error, "users-dismiss") {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Users));
    }

    let run = context.service::<Users>().run();
    if let Some(text) = texts::run(&ctx, run) {
        let tone = match run {
            PageRun::Failed(..) => Tone::Error,
            PageRun::Done(..) => Tone::Success,
            PageRun::None | PageRun::Sent(..) | PageRun::Running(..) => Tone::Neutral,
        };

        let console = i18n::text(&ctx, "users-show-console");
        match banner(ui, tone, &text, &[&dismiss, &console]).value {
            Some(0) => context.commands.send(UsersCommand::DismissRun),
            Some(_) => context.commands.send(UsersCommand::ShowConsole),
            None => {}
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    // Locks belong to accounts, so the Groups tab does not repeat it.
    let locks = context.service::<Users>().locks();
    if tab == Tab::Users
        && let Some(text) = texts::locks(&ctx, locks)
    {
        let again = i18n::text(&ctx, "users-refresh");
        let actions: &[&str] = if matches!(locks, Locks::Loading) {
            &[]
        } else {
            &[&again]
        };

        if banner(ui, Tone::Neutral, &text, actions).value == Some(0) {
            context.commands.send(UsersCommand::Refresh);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}
