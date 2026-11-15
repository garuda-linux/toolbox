//! What the app takes in from its services each frame: the apply
//! service's reports, the snapshot and what the page services took in.

use std::sync::Arc;

use toolbox_core::Snapshot;
use toolbox_core::domains::{AccountLockFlags, BootLoader, CurrentLoader};
use toolbox_core::features::Guard;
use toolbox_core::features::packages::aur::AurNames;
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_plan::StepId;
use toolbox_plan::protocol::Mode;
use toolbox_services::locks::Locks;
use toolbox_services::unprobed::{Needs, UnprobedReads, joined};
use toolbox_system::Notification;

use super::App;
use crate::apply::{ApplyCommand, Report};
use crate::console::StepState;
use crate::i18n;
use crate::network::Read;
use crate::page_service::Services;

/// The program output one frame feeds to the Console's terminal. A
/// program that prints faster than that still leaves the window usable.
const REPORT_BUDGET: usize = 128 * 1024;

impl App {
    /// A bubble only while the user looks elsewhere. The Console says the
    /// same to a user who watches.
    fn notify_finished(&self, ctx: &egui::Context, succeeded: bool) {
        let Some(notifier) = &self.notifier else {
            return;
        };

        if ctx.input(looked_at) {
            return;
        }

        let (summary, body) = if succeeded {
            ("notify-applied", "notify-applied-body")
        } else {
            ("notify-failed", "notify-failed-body")
        };

        notifier.show(Notification {
            summary: i18n::text(ctx, summary).into_owned(),
            body: i18n::text(ctx, body).into_owned(),
        });
    }

    pub(super) fn take_reports(&mut self, ctx: &egui::Context) {
        let (reports, more) = self
            .apply
            .as_ref()
            .map(|apply| apply.reports_within(REPORT_BUDGET))
            .unwrap_or_default();

        if more {
            ctx.request_repaint();
        }

        for report in reports {
            let finished = match report {
                Report::PlanStarted { plan, steps: _ } => {
                    self.services_plan_started(ctx, plan);
                    false
                }
                Report::Busy { plan } => {
                    self.services_plan_refused(ctx, plan);
                    false
                }
                Report::Step(_) | Report::Problem(_) | Report::CancelWaits => false,
                Report::PlanFinished { plan: _, succeeded } => {
                    self.plan_finished(ctx, succeeded);
                    true
                }
            };

            self.console.take(report);
            if finished {
                self.plans.last_ending = self.console.ending(Mode::of_dry_run(self.dry_run));
            }
        }

        // Answers to the program's own queries, such as the cursor position.
        let replies = self.console.terminal_mut().take_replies();
        if !replies.is_empty() {
            self.to_service(ApplyCommand::Input(replies));
        }
    }

    /// Every part learns that a plan ended, and the system is read again.
    fn plan_finished(&mut self, ctx: &egui::Context, succeeded: bool) {
        // A dry run skips every step, so nothing it planned counts as done
        // and no change learns that it failed.
        let steps = self.console.steps();
        let ended = |id: StepId, well: fn(&StepState) -> bool| {
            steps.iter().any(|step| step.id == id && well(&step.state))
        };

        let ended_well = |id: StepId| {
            ended(id, |state| {
                matches!(state, StepState::Succeeded | StepState::Skipped)
            })
        };

        let well: &dyn Fn(StepId) -> bool = &ended_well;

        self.queue.plan_finished(
            |id| ended(id, |state| *state == StepState::Succeeded),
            (!self.dry_run).then_some(well),
        );

        self.plans.finished += 1;
        self.plans.last_succeeded = succeeded;
        self.services_plan_finished(ctx, succeeded);
        self.close_after_run(ctx);

        self.notify_finished(ctx, succeeded);
        self.toast_finished(ctx, succeeded);

        // A failed plan may have done part of its work too.
        self.read_system();
    }

    /// Reads the system again when the window comes back to the front, for
    /// changes made outside the app meanwhile, and lets the queue meet each
    /// new snapshot once.
    pub(super) fn take_reading(&mut self, ctx: &egui::Context) {
        let focused = ctx.input(looked_at);
        if focused && !self.focused {
            self.read_system();
        }

        self.focused = focused;

        let Some(snapshot) = self.snapshot.as_mut() else {
            return;
        };

        let Some(reading) = snapshot.changed() else {
            return;
        };

        let fresh = reading.latest().is_some_and(|probed| {
            self.reading
                .latest()
                .is_none_or(|old| !Arc::ptr_eq(old, probed))
        });

        self.reading = reading;
        self.combine();
        if fresh {
            self.queue.meet(&self.seen);

            // An apply may have changed what /etc/skel holds.
            self.services.maintenance.restock();
        }
    }

    pub(super) fn first_read(&self) -> bool {
        self.snapshot.is_some() && self.reading.get().is_none()
    }

    /// Why Apply waits, as the id of its text. A guard that has not
    /// checked a restored change fails closed. A read that failed says
    /// so, as the Changes popover then offers to read it again.
    pub(super) fn apply_held(&self) -> Option<&'static str> {
        if self.first_read() {
            return Some("changes-wait-read");
        }

        self.queue.holding().next().map(|guard| match guard {
            Guard::UserLocks => match self.services.users.locks() {
                Locks::Refused(_) | Locks::Unknown => "changes-unread-locks",
                Locks::NotAsked | Locks::Loading | Locks::Known(_) => "changes-wait-locks",
            },
            Guard::AurPackages => match self.services.packages.aur() {
                Read::Failed => "changes-unread-aur",
                Read::Loading | Read::Unavailable | Read::Known(_) => "changes-wait-aur",
            },
            Guard::Users
            | Guard::Memberships
            | Guard::Kernels
            | Guard::Locales
            | Guard::LanguagePacks
            | Guard::PackageRemovals
            | Guard::Plymouth
            | Guard::Grub
            | Guard::ModuleBlacklist
            | Guard::NewGroups
            | Guard::Hblock
            | Guard::WifiBackend => "changes-wait-read",
        })
    }

    /// No probe reads the lock flags, the AUR's names or the loader's
    /// settings, so a restored change that waits for them would wait until
    /// its page is opened.
    fn read_held_parts(&mut self) {
        let needs = Needs::of(self.queue.changes(), self.queue.holding());
        if needs.locks {
            self.services.users.open();
        }

        if needs.aur {
            self.services.packages.open_aur();
        }

        if needs.loader {
            let sdboot = self.seen.part::<BootLoader>() == Some(&BootLoader::SystemdBoot);
            self.services.boot.open(sdboot);
        }
    }

    pub(super) fn read_system(&self) {
        if let Some(snapshot) = &self.snapshot {
            snapshot.refresh();
        }
    }

    /// The latest snapshot joined by [`Self::combine`], or an empty one
    /// before the first read.
    pub(super) fn current_snapshot(&self) -> Arc<Snapshot> {
        Arc::clone(&self.seen)
    }

    /// Joins the probed snapshot, the lock flags, the AUR's names and the
    /// settings of systemd-boot. A page keeps the snapshot it was handed,
    /// so this makes a new one.
    pub(super) fn combine(&mut self) {
        let probed = self
            .reading
            .get()
            .map_or_else(Arc::default, |probed| Arc::clone(&probed.snapshot));

        let reads = PageReads(&self.services);
        self.seen = joined(&probed, Needs::ALL, &reads).map_or(probed, Arc::new);
    }

    pub(super) fn take_services(&mut self, ctx: &egui::Context) {
        self.take_page_services(ctx);
        self.take_output_share(ctx);
        self.read_held_parts();

        // The writer logged why, the toast says that a restart loses it.
        if self.queue.not_kept().is_some() {
            self.toasts
                .add(ToastKind::Error, i18n::text(ctx, "toast-queue-not-saved"));
        }
    }

    /// A run ended while the Console was hidden, so nothing on screen says
    /// how it went.
    fn toast_finished(&mut self, ctx: &egui::Context, succeeded: bool) {
        if self.state.console_open {
            return;
        }

        let (kind, id) = if succeeded {
            (ToastKind::Info, "toast-plan-succeeded")
        } else {
            (ToastKind::Error, "toast-plan-failed")
        };

        self.toasts.add(kind, i18n::text(ctx, id));
    }
}

/// What the pages' services read last. Every read is cached, so the window
/// joins them all.
struct PageReads<'a>(&'a Services);

impl UnprobedReads for PageReads<'_> {
    fn locks(&self) -> Option<AccountLockFlags> {
        self.0.users.flags()
    }

    fn aur_names(&self) -> Option<AurNames> {
        self.0.packages.aur_names().cloned()
    }

    fn loader(&self) -> Option<CurrentLoader> {
        self.0
            .boot
            .menu()
            .sdboot()
            .map(|menu| CurrentLoader(menu.settings.clone()))
    }
}

/// While the window is hidden eframe runs no pass, so `input.focused`
/// stays that of the last shown frame. The raw input and the viewport are
/// current.
pub(super) fn looked_at(input: &egui::InputState) -> bool {
    let viewport = input.viewport();
    input.raw.focused && viewport.minimized != Some(true) && viewport.occluded != Some(true)
}
