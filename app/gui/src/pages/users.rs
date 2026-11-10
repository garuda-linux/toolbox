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
use toolbox_core::features::users::run::PageRun;
use toolbox_core::features::users::view::{self, Shown};
use toolbox_gui_kit::widgets::{ButtonKind, Tone, banner, empty_state, page_toolbar, pill_button};
use toolbox_gui_kit::{UiExt as _, icons};

use self::dialogs::{Dialog, Next};
use super::header::{TabText, TabbedIds, search, tabbed_header};
use super::rows::refusal_banner;
use super::{PageContext, PageView, plan_running};
use crate::command::UsersCommand;
use crate::i18n;
use crate::users::Users;

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
}

impl Default for UsersPage {
    fn default() -> Self {
        Self {
            tab: Tab::default(),
            shown: Shown::people(),
            dialog: None,
        }
    }
}

impl PageView for UsersPage {
    /// A dialog may hold a password, which must not wait for the return.
    fn left(&mut self, ctx: &egui::Context) {
        if let Some(mut dialog) = self.dialog.take() {
            dialog.forget(ctx);
        }
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

        let done = tabbed_header(ui, ids, &[Tab::Users, Tab::Groups], self.tab);
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
        notes(ui, context);
        plan_running(ui, context);
        match self.tab {
            Tab::Users => list::show(ui, context, &mut self.shown, &mut self.dialog),
            Tab::Groups => groups::show(ui, context, &mut self.shown, &mut self.dialog),
        }

        let ctx = ui.ctx().clone();
        if let Some(dialog) = &mut self.dialog
            && dialog.show(&ctx, context) == Next::Close
        {
            dialog.forget(&ctx);
            self.dialog = None;
        }
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
}

/// Whether the accounts were read, else the empty state that says why.
pub fn accounts_ready(
    ui: &mut Ui,
    context: &PageContext<'_>,
    reading_id: &str,
    unavailable_id: &str,
) -> bool {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    if !context.probed {
        empty_state(ui, icons::USER_OTHERS, &text(reading_id), "");
        return false;
    }

    if !view::accounts_known(context.snapshot) {
        let detail = text("users-unavailable-detail");
        empty_state(ui, icons::HELP_ABOUT, &text(unavailable_id), &detail);
        return false;
    }

    true
}

fn notes(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let dismiss = i18n::text(&ctx, "users-dismiss");
    if let Some(refusal) = &context.refusals.users {
        let text = texts::problem(&ctx, refusal.code, &refusal.subject);
        if refusal_banner(ui, Tone::Error, &text, "users-dismiss") {
            context.commands.send(UsersCommand::DismissRefusal);
        }
    }

    let run = context.service::<Users>().run();
    if let Some(text) = texts::run(&ctx, run) {
        let tone = match run {
            PageRun::Failed(..) => Tone::Error,
            PageRun::Done(..) => Tone::Success,
            PageRun::None | PageRun::Sent(..) | PageRun::Running(..) => Tone::Neutral,
        };

        if banner(ui, tone, &text, &[&dismiss]) == Some(0) {
            context.commands.send(UsersCommand::DismissRun);
        }

        let console = i18n::text(&ctx, "users-show-console");
        if pill_button(ui, &console, ButtonKind::Soft).clicked() {
            context.commands.send(UsersCommand::ShowConsole);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    if let Some(text) = texts::locks(&ctx, context.service::<Users>().locks()) {
        banner(ui, Tone::Neutral, &text, &[]);
        ui.add_space(ui.tokens().sizes.card_gap);
    }
}
