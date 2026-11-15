//! What the app does with a command from the bus or a shortcut.

use toolbox_plan::Plan;

use super::App;
use crate::apply::{ApplyCommand, PlanId};
use crate::command::{AppCommand, Command, DiscoverCommand, PackagesCommand, Sent, UiCommand};
use crate::page::PageId;
use crate::page_service::Owner;
use crate::page_ui;

impl App {
    /// Runs every command that reached the app this frame: the shortcuts
    /// first, then what was sent.
    pub(super) fn run_all(&mut self, ctx: &egui::Context, pressed: Vec<UiCommand>) {
        for command in pressed {
            let command = Command::Keyed(command);
            let _command = tracing::debug_span!(
                "command",
                command = %command.for_log(),
                from = "shortcut"
            )
            .entered();

            self.run(ctx, command);
        }

        let sent: Vec<Sent> = self.receiver.drain().collect();
        for Sent { command, from } in sent {
            // `for_log` hides what is typed into the Console.
            let _command =
                tracing::debug_span!("command", command = %command.for_log(), from = %from)
                    .entered();

            self.run(ctx, command);
        }
    }

    /// The one place a destructive command waits for the user's yes.
    pub(super) fn run(&mut self, ctx: &egui::Context, command: Command) {
        match command {
            Command::Ui(command) if command.is_destructive() => {
                self.changes_ui.confirming = Some(command);
            }
            Command::Ui(command) | Command::Confirmed(command) => self.run_ui(ctx, command),
            Command::Keyed(command) => {
                self.run(ctx, Command::Ui(command));

                // What a key toggled jumps, what the pointer toggled slides.
                self.snap.sidebar |= command == UiCommand::ToggleSidebar;
                self.snap.console |= command == UiCommand::ToggleConsole;
                self.snap.page |= matches!(command, UiCommand::Navigate(_));
                self.changes_ui.keyed |= command == UiCommand::ReviewChanges;
            }
            Command::App(AppCommand::Setting(setting)) => self.run_setting(ctx, setting),
            Command::App(AppCommand::Activated(arrival)) => self.arrive(ctx, &arrival),
            Command::App(command) => self.run_app(ctx, command),
        }
    }

    pub(super) fn run_ui(&mut self, ctx: &egui::Context, command: UiCommand) {
        match command {
            UiCommand::Navigate(page) => self.navigate(ctx, page),
            UiCommand::ToggleSidebar => self.state.sidebar_expanded ^= true,
            UiCommand::ToggleConsole => self.state.console_open ^= true,
            UiCommand::OpenPalette => self.palette.open(),
            UiCommand::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            UiCommand::ClearChanges => self.queue.clear(),
            UiCommand::Apply => {
                if self.apply_queue() {
                    crate::shell::focus_console_on_open(ctx);
                }
            }
            UiCommand::ReviewChanges => self.changes_ui.open = true,
            UiCommand::FocusSearch => toolbox_gui_kit::widgets::request_search_focus(ctx),
            UiCommand::Help => {
                let (file, anchor) = self.state.page.guide_topic();
                self.pages.show_guide(file, anchor);
                self.run_ui(ctx, UiCommand::Navigate(PageId::Guide));
            }
        }
    }

    /// The one way to another page, also for a page or a service that asks
    /// for one, so the old page hears it was left.
    pub(super) fn navigate(&mut self, ctx: &egui::Context, page: PageId) {
        // A shortcut, an old setting or a service may name a page this
        // system hides.
        if !page_ui::listed(ctx, page) {
            return;
        }

        if page != self.state.page {
            self.pages.view(self.state.page).left(ctx);

            // A finished run's Console would cover three quarters of the
            // next page. A running one stays, its output matters.
            let finished =
                !self.console.running() && self.console.status() != crate::console::Status::Idle;

            if finished {
                self.state.console_open = false;
            }
        }

        self.state.page = page;
    }

    pub(super) fn run_app(&mut self, ctx: &egui::Context, command: AppCommand) {
        let (taken, asked) = self.hand_to_services(&command);
        if !taken {
            self.run_own(ctx, command);
        }

        for next in asked {
            self.run_app(ctx, next);
        }
    }

    /// App's part of `command`, after the page services had theirs.
    fn run_own(&mut self, ctx: &egui::Context, command: AppCommand) {
        match command {
            AppCommand::RemoveChange { domain, key } => {
                self.queue.remove(domain, &key);
            }
            AppCommand::CancelPlan => self.to_service(ApplyCommand::Cancel),
            AppCommand::ConsoleInput(bytes) => {
                self.to_service(ApplyCommand::Input(bytes));
            }
            AppCommand::ConsoleResize(size) => {
                self.to_service(ApplyCommand::Resize(size));
            }
            AppCommand::ReadSystem => self.read_system(),
            // A page service takes these, see `hand_to_services`.
            AppCommand::ToggleEntry {
                catalog: _,
                entry: _,
                wish: _,
            }
            | AppCommand::DismissRefusal(_)
            | AppCommand::Kernel(_)
            | AppCommand::Boot(_)
            | AppCommand::Locale(_)
            | AppCommand::Tweak(_)
            | AppCommand::Diagnostics(_) => {
                tracing::warn!("no page service took {command:?}");
            }
            AppCommand::Toast { kind, text } => self.toasts.add(kind, text),
            AppCommand::Home(home) => self.run_home(home),
            AppCommand::Service(service) => self.run_service(service),
            AppCommand::Discover(discover) => self.run_discover(ctx, discover),
            AppCommand::Maintenance(maintenance) => self.run_maintenance(ctx, &maintenance),
            AppCommand::Network(network) => self.run_network(ctx, network),
            AppCommand::Packages(packages) => self.run_packages(&packages),
            AppCommand::Users(users) => self.run_users(users),
            AppCommand::Configs(configs) => self.run_configs(&configs),
            AppCommand::BootRepair(repair) => self.run_boot_repair(repair),
            AppCommand::Console(console) => self.run_console(console),
            AppCommand::Answer(answer) => self.answer(ctx, answer),
            // `run` takes these first, as they need the window.
            AppCommand::Activated(arrival) => {
                tracing::warn!("an activation missed its turn: {arrival:?}");
            }
            AppCommand::Setting(setting) => {
                tracing::warn!("a setting missed its turn: {setting:?}");
            }
        }
    }

    /// Runs `plan` past the queue and opens the Console, which shows the
    /// run. A plan that runs already keeps it, and the buttons wait for its
    /// end. Returns the id `plan` was sent under, `None` when it was not.
    pub(super) fn run_at_once(&mut self, plan: Plan) -> Option<PlanId> {
        let id = self.send_plan(plan)?;
        self.show_run();
        Some(id)
    }

    /// [`Self::run_at_once`] for a plan whose start `owner` waits for.
    /// Returns whether `plan` was sent.
    pub(super) fn run_owned(&mut self, plan: Plan, owner: Owner) -> bool {
        let sent = self.run_unseen(plan, owner);
        if sent {
            self.show_run();
        }

        sent
    }

    /// Runs `plan` past the queue with the Console closed, for a page that
    /// shows the outcome itself and waits for the start as `owner`.
    /// Returns whether `plan` was sent.
    pub(super) fn run_unseen(&mut self, plan: Plan, owner: Owner) -> bool {
        let Some(id) = self.send_plan(plan) else {
            return false;
        };

        self.services.waiting.sent(id, owner);
        true
    }

    /// `None` while another plan runs, which keeps the Console.
    fn send_plan(&mut self, plan: Plan) -> Option<PlanId> {
        if self.console.running() {
            return None;
        }

        let id = self.console.plan_sent(&plan);
        self.to_service(ApplyCommand::Run { id, plan });
        Some(id)
    }

    /// The list of changes would only cover the run.
    const fn show_run(&mut self) {
        self.state.console_open = true;
        self.changes_ui.open = false;
    }

    /// Opens the Console, so the password dialog and the output are seen.
    /// Waits until every page's guard checked a restored queue, so it never
    /// runs what a page refuses. Returns whether the run started.
    fn apply_queue(&mut self) -> bool {
        let Ok(plan) = self.queue.plan() else {
            return false;
        };

        if plan.is_empty() || self.first_read() || self.console.running() {
            return false;
        }

        self.queue.sent(&plan);
        self.run_owned(plan, Owner::Queue)
    }

    pub(super) fn to_service(&self, command: ApplyCommand) {
        if let Some(apply) = &self.apply {
            apply.send(command);
        }
    }
}

/// The clicks of Discover and Packages that need App beyond their
/// services: a reveal opens Discover, and Refresh on Packages also reads
/// Home's status and facts.
impl App {
    pub(super) fn run_discover(&mut self, ctx: &egui::Context, command: DiscoverCommand) {
        match command {
            DiscoverCommand::Reveal(title) => {
                self.pages.reveal_app(&title);
                self.navigate(ctx, PageId::Discover);
            }
            // `TogglePages` takes these, see `hand_to_services`.
            DiscoverCommand::Toggle { app: _, wish: _ } => {
                tracing::warn!("no page service took {command:?}");
            }
        }
    }

    pub(super) fn run_packages(&mut self, command: &PackagesCommand) {
        match command {
            PackagesCommand::Refresh => {
                self.services.packages.refresh();
                self.read_system();
                self.services.home.refresh_packages();
            }
            // The service takes these, see `hand_to_services`.
            PackagesCommand::Show(_)
            | PackagesCommand::OpenAur
            | PackagesCommand::Toggle {
                package: _,
                wish: _,
            }
            | PackagesCommand::ToggleAur {
                package: _,
                wish: _,
            } => {}
        }
    }
}
