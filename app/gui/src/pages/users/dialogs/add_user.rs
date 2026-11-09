//! Creates a user and sets its password at once. Each field says what is
//! wrong with it while it is typed. Core builds the account from the
//! fields, so the dialog and the queue apply the same rules.

use std::collections::BTreeSet;
use std::fmt;

use egui::{Context, TextEdit, Ui};
use toolbox_core::features::users::home::check_uid;
use toolbox_core::features::users::input::{self, AddForm, NewAccount, add_blocker};
use toolbox_core::features::users::now::check_new_name;
use toolbox_core::features::users::{Refusal, view};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    ButtonKind, CHECK_ROW_HEIGHT, Select, check_row, disclosure, text_field,
};
use toolbox_plan::{FullName, GroupName, InvalidFullName, NewUser, ShellPath, UserName};

use super::password::PasswordPair;
use super::{Next, dialog_why, field, note, title};
use crate::command::UsersCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::users::texts;

/// The groups the Add user list shows before it scrolls.
const GROUPS_SHOWN: f32 = 5.5;

#[derive(Default)]
pub struct AddUser {
    name: String,
    full_name: String,
    /// Empty for the system's default shell.
    shell: String,
    home: String,
    uid: String,
    no_home: bool,
    system: bool,
    pub(super) password: PasswordPair,
    admin: bool,
    groups: BTreeSet<GroupName>,
    search: String,
}

/// What each field says is wrong with it, `None` where nothing is.
#[derive(Default)]
struct Problems {
    name: Option<String>,
    full_name: Option<String>,
    home: Option<String>,
    uid: Option<String>,
}

impl fmt::Debug for AddUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddUser")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl AddUser {
    pub fn show(&mut self, ctx: &Context, context: &PageContext<'_>) -> Next {
        let title = i18n::text(ctx, "users-add-title");
        let confirm = i18n::text(ctx, "users-add-confirm");
        let (account, problems) = self.check(ctx, context);
        let agreed = self.password.agreed();
        let ready = account.is_ok() && agreed && !context.busy;
        let why = self.why(ctx, context, &problems, agreed);
        let (next, confirmed) = dialog_why(
            ctx,
            "add-user",
            (&title, &confirm, ButtonKind::Primary),
            (why.as_deref(), ready),
            |ui| {
                self.required(ui, ctx, context, &problems);

                // Kept by egui, so the dialog opens again as it was left.
                let id = egui::Id::unique("toolbox_gui::users::add_more");
                let mut open = ui.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
                let more = i18n::text(ctx, "users-add-more");
                if disclosure(ui, &more, open).clicked() {
                    open = !open;
                    ui.data_mut(|data| data.insert_temp(id, open));
                }

                if open {
                    ui.add_space(ui.tokens().sizes.card_gap);
                    self.identity(ui, ctx, context, &problems);
                    self.place(ui, ctx, &problems);
                    self.other_groups(ui, ctx, context);
                }

                note(ui, &i18n::text(ctx, "users-add-note"));
            },
        );

        if confirmed && let (Ok(user), Some(password)) = (account, self.password.password()) {
            context.commands.send(UsersCommand::AddUser {
                user: Box::new(user),
                password,
            });
        }

        next
    }

    /// The account the fields make, and each field's problem.
    fn check(
        &self,
        ctx: &Context,
        context: &PageContext<'_>,
    ) -> (Result<NewUser, &'static str>, Problems) {
        let snapshot = context.snapshot;
        let named = |code: &str| texts::problem(ctx, code, &self.name);
        let name = input::name(&self.name)
            .and_then(|user: UserName| check_new_name(&user, snapshot).map_err(Refusal::as_str));

        let uid = input::user_id(&self.uid)
            .and_then(|uid| check_uid(uid, self.system, snapshot).map_err(Refusal::as_str));

        let problems = Problems {
            name: name.err().filter(|_| !self.name.is_empty()).map(named),
            full_name: FullName::new(self.full_name.clone())
                .map_err(InvalidFullName::as_str)
                .err()
                .map(|code| texts::full_name_problem(ctx, code)),
            home: (!self.no_home && !self.home.is_empty())
                .then(|| input::home(&self.home).err())
                .flatten()
                .map(named),
            uid: uid.err().map(named),
        };

        let mut groups = self.groups.clone();
        if self.admin
            && let Some(admin) = view::admin_group(snapshot, context.changes)
        {
            groups.insert(admin.clone());
        }

        let form = NewAccount {
            name: &self.name,
            full_name: &self.full_name,
            shell: &self.shell,
            groups: Some(groups),
            home: &self.home,
            no_home: self.no_home,
            uid: &self.uid,
            system: self.system,
        };

        (input::new_account(&form, snapshot), problems)
    }

    /// Why "Add user now" waits, `None` once it can be clicked.
    fn why(
        &self,
        ctx: &Context,
        context: &PageContext<'_>,
        problems: &Problems,
        password_ok: bool,
    ) -> Option<String> {
        let (first, second) = self.password.texts();
        let form = AddForm {
            name: &self.name,
            name_ok: problems.name.is_none(),
            first,
            second,
            password_ok,
            rest_ok: problems.full_name.is_none()
                && problems.home.is_none()
                && problems.uid.is_none(),
        };

        let code = match add_blocker(&form) {
            Some(blocker) => blocker.as_str(),
            None if context.busy => "busy",
            None => return None,
        };

        Some(i18n::text(ctx, &format!("users-add-why-{code}")).into_owned())
    }

    /// The name, the password and whether the account may administer.
    fn required(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        context: &PageContext<'_>,
        problems: &Problems,
    ) {
        let text = |id: &str| i18n::text(ctx, id).into_owned();
        let labels = (text("users-field-name"), text("users-field-name-hint"));
        field(
            ui,
            (&labels.0, &labels.1),
            &mut self.name,
            problems.name.as_deref(),
        );
        self.password.show(ui, ctx);

        if let Some(group) = view::admin_group(context.snapshot, context.changes) {
            let label = text("users-admin");
            let detail = i18n::text_with(ctx, "users-admin-detail", &[("group", group.as_str())]);
            if check_row(ui, self.admin, &label, &detail, true).clicked() {
                self.admin = !self.admin;
            }

            note(ui, &detail);
            ui.add_space(ui.tokens().sizes.card_gap);
        }
    }

    fn identity(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        context: &PageContext<'_>,
        problems: &Problems,
    ) {
        let text = |id: &str| i18n::text(ctx, id).into_owned();
        let full_name = text("users-field-full-name");
        field(
            ui,
            (&full_name, ""),
            &mut self.full_name,
            problems.full_name.as_deref(),
        );

        shell_choice(ui, ctx, context, &mut self.shell);
    }

    fn place(&mut self, ui: &mut Ui, ctx: &Context, problems: &Problems) {
        let text = |id: &str| i18n::text(ctx, id).into_owned();
        if !self.no_home {
            let labels = (text("users-field-home"), text("users-field-home-hint"));
            field(
                ui,
                (&labels.0, &labels.1),
                &mut self.home,
                problems.home.as_deref(),
            );
        }

        let labels = (text("users-field-uid"), text("users-field-uid-hint"));
        field(
            ui,
            (&labels.0, &labels.1),
            &mut self.uid,
            problems.uid.as_deref(),
        );
        for (on, id) in [
            (&mut self.no_home, "users-no-home"),
            (&mut self.system, "users-system"),
        ] {
            let detail = text(&format!("{id}-detail"));
            if check_row(ui, *on, &text(id), &detail, true).clicked() {
                *on = !*on;
            }

            note(ui, &detail);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }

    /// The groups besides the administrators.
    fn other_groups(&mut self, ui: &mut Ui, ctx: &Context, context: &PageContext<'_>) {
        let admin = view::admin_group(context.snapshot, context.changes);
        title(ui, &i18n::text(ctx, "users-other-groups"));
        let search = text_field(
            ui,
            TextEdit::singleline(&mut self.search)
                .hint_text(i18n::text(ctx, "users-groups-search-hint"))
                .desired_width(f32::INFINITY)
                .align(egui::Align2::LEFT_CENTER),
        );

        let name = i18n::text(ctx, "users-groups-search").into_owned();
        ui.ctx()
            .accesskit_node_builder(search.id, |node| node.set_label(name));

        let names = view::group_names(context.snapshot, context.changes, &self.search);

        // A system has dozens of groups. Half a row more than fits shows
        // that the list scrolls.
        let row = CHECK_ROW_HEIGHT + ui.spacing().item_spacing.y;
        let mut focused = None;
        let list = egui::ScrollArea::vertical()
            .id_salt("users-other-groups")
            .max_height(row * GROUPS_SHOWN)
            // Inside the dialog's own scrolling body egui would give the
            // list only its 64 point default.
            .min_scrolled_height(row * GROUPS_SHOWN)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for group in names.iter().filter(|group| Some(*group) != admin) {
                    let on = self.groups.contains(group);
                    let box_row = check_row(ui, on, group.as_str(), "", true);
                    if box_row.gained_focus() {
                        focused = Some(box_row.rect);
                    }

                    if box_row.clicked() {
                        if on {
                            self.groups.remove(group);
                        } else {
                            self.groups.insert(group.clone());
                        }
                    }
                }
            });

        // The list scrolls only itself to a box. The dialog around it must
        // bring the list into sight too, or Tab lands on a hidden box.
        if let Some(box_rect) = focused {
            ui.scroll_to_rect(box_rect.intersect(list.inner_rect), None);
        }

        ui.add_space(ui.tokens().sizes.card_gap);
    }
}

/// The login shell: the system's default first, then what `/etc/shells`
/// lists. Empty `chosen` is the default.
fn shell_choice(ui: &mut Ui, ctx: &Context, context: &PageContext<'_>, chosen: &mut String) {
    let text = |id: &str| i18n::text(ctx, id).into_owned();
    let shells = view::shells(context.snapshot);
    let default = text("users-shell-default");
    let shown = ShellPath::names(shells);
    let names: Vec<&str> = std::iter::once(default.as_str())
        .chain(shown.iter().map(String::as_str))
        .collect();

    let picked = shells.iter().position(|shell| shell.as_str() == chosen);
    let current = if chosen.is_empty() {
        Some(0)
    } else {
        picked.map(|index| index + 1)
    };

    let label = text("users-field-shell");
    title(ui, &label);
    if let Some(index) = Select::new(&label, &names)
        .selected(current)
        .detail(
            picked
                .and_then(|index| shells.get(index))
                .map(ShellPath::as_str),
        )
        .width(ui.available_width())
        .show(ui)
    {
        *chosen = index
            .checked_sub(1)
            .and_then(|index| shells.get(index))
            .map(|shell| shell.as_str().to_owned())
            .unwrap_or_default();
    }

    note(ui, &text("users-field-shell-hint"));
    ui.add_space(ui.tokens().sizes.card_gap);
}
