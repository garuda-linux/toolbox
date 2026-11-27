//! The build orders of the queued AUR builds, read off the frame. Apply
//! waits for a fresh read of every order, then plans and sends at once.

use std::borrow::Cow;
use std::sync::Arc;

use toolbox_core::domains::{AurPackages, AurWish, waiting_orders};
use toolbox_core::features::packages::aur::route_refusal;
use toolbox_services::aur_orders::{self, route_text_id};

use crate::app::App;
use crate::i18n::{self, Arg};
use crate::page_service::Owner;
use crate::shell::{BuildNote, Hold};

impl App {
    /// Asks for the build orders the queue needs. An apply that waits for a
    /// fresh read asks for every build, and sends its plan when that
    /// landed or runs nothing when no read can start.
    pub(in crate::app) fn ask_orders(&mut self) {
        if self.first_read() {
            return;
        }

        let snapshot = Arc::clone(&self.seen);
        let forced = self.services.packages.apply_revision().is_some();
        let packages = &mut self.services.packages;
        let started = packages.ask_orders(self.queue.changes(), &snapshot, forced);

        if forced && !started && !packages.reading_orders() {
            packages.end_apply();
            self.run_queue_plan();
        }
    }

    /// Applies a read of build orders that landed. An apply that waited for
    /// it sends its plan in this same pass, unless the queue changed since
    /// the apply was clicked.
    pub(in crate::app) fn take_orders(&mut self) {
        let snapshot = Arc::clone(&self.seen);
        let packages = &mut self.services.packages;
        let Some(landing) = packages.take_orders() else {
            return;
        };

        let forced = packages.orders_forced();
        let waiting = packages.apply_revision();
        let kept = waiting == Some(self.queue.revision());

        self.queue.edit(&snapshot, |changes, snapshot| {
            packages.land_orders(changes, snapshot, landing);
        });

        match (waiting, forced) {
            (Some(_), true) if kept => {
                packages.end_apply();
                self.run_queue_plan();
            }
            (Some(_), false) if kept => packages.rebase_apply(self.queue.revision()),
            (Some(_), _) => packages.end_apply(),
            (None, _) => {}
        }
    }

    /// Apply in the queue's window: a queue with AUR builds reads their
    /// orders first, a queue without them runs at once.
    pub(in crate::app) fn apply_queue(&mut self) -> bool {
        if self.first_read()
            || self.console.running()
            || self.services.packages.apply_revision().is_some()
        {
            return false;
        }

        if !aur_orders::needing(self.queue.changes()).is_empty() {
            self.services.packages.start_apply(self.queue.revision());
            self.ask_orders();
            return false;
        }

        self.run_queue_plan()
    }

    /// Plans the queue and runs the plan. `false` when the queue refuses or
    /// is empty.
    pub(in crate::app) fn run_queue_plan(&mut self) -> bool {
        let Ok(plan) = self.queue.plan() else {
            return false;
        };

        if plan.is_empty() || self.console.running() {
            return false;
        }

        self.queue.sent(&plan);
        self.run_owned(plan, Owner::Queue)
    }

    /// A waiting Apply is dropped once the popover closed, by Review, Escape
    /// or a click beside it. The Apply that was clicked from the footer
    /// with the popover shut is kept.
    pub(in crate::app) const fn drop_dismissed_apply(&mut self) {
        if self.changes_ui.take_dismissed() {
            self.services.packages.end_apply();
        }
    }

    /// Why Apply waits, as a sentence that names the packages, and what each
    /// queued AUR build shows in the popover.
    pub(in crate::app) fn holds(
        &self,
        ctx: &egui::Context,
    ) -> (Option<Hold>, Vec<(String, BuildNote)>) {
        let hold = self.apply_held().map(|id| self.hold_of(ctx, id));
        (hold, self.build_notes())
    }

    fn hold_of(&self, ctx: &egui::Context, id: &'static str) -> Hold {
        let text = match id {
            "changes-wait-reading" => {
                let names = self.queued_names();
                i18n::text_with(ctx, id, &[("names", Arg::from(names.as_str()))])
            }
            "changes-wait-orders" => {
                let reason = self.order_reason(ctx);
                i18n::text_with(ctx, id, &[("reason", Arg::from(&*reason))])
            }
            _ => i18n::text(ctx, id),
        };

        Hold {
            id,
            text: text.into_owned(),
        }
    }

    /// The names of the queued AUR builds, comma separated.
    fn queued_names(&self) -> String {
        aur_orders::needing(self.queue.changes())
            .iter()
            .map(toolbox_plan::PackageName::as_str)
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Why the first build waits for its order: the reason of a read that
    /// failed, or the name of a build no read has answered yet.
    fn order_reason(&self, ctx: &egui::Context) -> Cow<'static, str> {
        if let Some((name, reason)) = self.services.packages.order_missing().first() {
            return i18n::text_with(ctx, reason.text_id(), &[("name", Arg::from(name.as_str()))]);
        }

        let waiting = waiting_orders(self.queue.changes());
        let name = waiting
            .first()
            .map(toolbox_plan::PackageName::as_str)
            .unwrap_or_default();

        i18n::text_with(
            ctx,
            "changes-wait-order-unknown",
            &[("name", Arg::from(name))],
        )
    }

    fn build_notes(&self) -> Vec<(String, BuildNote)> {
        let packages = &self.services.packages;
        let route = route_refusal(&self.seen).map_or("aur-route-other", route_text_id);

        self.queue
            .changes()
            .queued::<AurPackages>()
            .into_iter()
            .map(|(name, wish)| {
                let missing = packages
                    .order_missing()
                    .iter()
                    .find(|(missing, _)| *missing == name)
                    .map(|(_, reason)| reason.text_id());

                let note = match wish {
                    AurWish::Paru => BuildNote::Old(route),
                    AurWish::Waiting if packages.reading_orders() => BuildNote::Reading,
                    AurWish::Waiting => missing.map_or(BuildNote::Waiting, BuildNote::Unread),
                    AurWish::Built(order) => BuildNote::Ready(order.outline()),
                };

                (name.as_str().to_owned(), note)
            })
            .collect()
    }
}
