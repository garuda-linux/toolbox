//! What the app does with a command from the bus or a shortcut.

use toolbox_core::Toggle;
use toolbox_core::features::toggles;
use toolbox_plan::Plan;

use super::App;
use crate::apply::ApplyCommand;
use crate::catalogs::CatalogId;
use crate::command::{AppCommand, Command, Sent, UiCommand};
use crate::page::PageId;
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
            // A shortcut or an old setting may name a page this system hides.
            UiCommand::Navigate(page) if !page_ui::listed(ctx, page) => {}
            UiCommand::Navigate(page) => {
                // A finished run's Console would cover three quarters of
                // the next page. A running one stays, its output matters.
                let finished = !self.console.running()
                    && self.console.status() != crate::console::Status::Idle;

                if page != self.state.page {
                    self.pages.view(self.state.page).left(ctx);

                    if finished {
                        self.state.console_open = false;
                    }
                }

                self.state.page = page;
            }
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

    fn run_app(&mut self, ctx: &egui::Context, command: AppCommand) {
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
            AppCommand::ToggleEntry {
                catalog,
                entry,
                wish,
            } => self.toggle_entry(catalog, &entry, wish),
            AppCommand::DismissRefusal(catalog) => {
                drop(self.refusals.catalogs.remove(&catalog));
            }
            AppCommand::Toast { kind, text } => self.toasts.add(kind, text),
            AppCommand::Kernel(kernel) => self.run_kernel(kernel),
            AppCommand::Home(home) => self.run_home(home),
            AppCommand::Tweak(tweak) => self.tweak(tweak),
            AppCommand::Service(service) => self.run_service(service),
            AppCommand::Discover(discover) => self.run_discover(discover),
            AppCommand::Maintenance(maintenance) => self.run_maintenance(maintenance),
            AppCommand::Diagnostics(diagnostics) => self.run_diagnostics(diagnostics),
            AppCommand::Boot(boot) => self.run_boot(boot),
            AppCommand::Locale(locale) => self.run_locale(locale),
            AppCommand::Network(network) => self.run_network(ctx, network),
            AppCommand::Packages(packages) => self.run_packages(packages),
            AppCommand::Users(users) => self.run_users(users),
            AppCommand::Configs(configs) => self.run_configs(configs),
            AppCommand::BootRepair(repair) => self.run_boot_repair(repair),
            AppCommand::Console(console) => self.run_console(console),
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
    /// end. Returns whether `plan` was sent.
    pub(super) fn run_at_once(&mut self, plan: Plan) -> bool {
        if self.console.running() {
            return false;
        }

        self.state.console_open = true;
        self.changes_ui.open = false;
        self.console.plan_sent(&plan);
        self.to_service(ApplyCommand::Run(plan));
        true
    }

    /// Runs `plan` past the queue with the Console closed, for a page that
    /// shows the outcome itself. Returns whether `plan` was sent.
    pub(super) fn run_unseen(&mut self, plan: Plan) -> bool {
        if self.console.running() {
            return false;
        }

        self.console.plan_sent(&plan);
        self.to_service(ApplyCommand::Run(plan));
        true
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

        self.state.console_open = true;

        // The Console shows the run, the list would only cover it.
        self.changes_ui.open = false;
        self.console.plan_sent(&plan);
        self.queue.sent(&plan);
        self.to_service(ApplyCommand::Run(plan));
        true
    }

    pub(super) fn to_service(&self, command: ApplyCommand) {
        if let Some(apply) = &self.apply {
            apply.send(command);
        }
    }

    /// The refusal stays on its page until dismissed or a later click on
    /// that page is queued.
    fn toggle_entry(&mut self, catalog: CatalogId, entry: &str, wish: Toggle) {
        let Ok(toggle_catalog) = &self.catalogs.get(catalog).catalog else {
            return;
        };

        let snapshot = self.current_snapshot();
        let result = self.queue.edit(&snapshot, |changes, snapshot| {
            toggles::toggle(toggle_catalog, entry, wish, snapshot, changes)
        });

        match result {
            Ok(()) => drop(self.refusals.catalogs.remove(&catalog)),
            Err(refusal) => drop(self.refusals.catalogs.insert(catalog, refusal)),
        }
    }
}
