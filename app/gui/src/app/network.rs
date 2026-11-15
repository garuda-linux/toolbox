//! Driver loads, service starts and the tests run at once in the Console. The
//! service takes the other clicks.

use std::path::Path;

use toolbox_core::features::network::{
    HOTSPOT_PROGRAM, Refusal, plan_load, plan_ping, plan_start, plan_traceroute, plan_unload,
};
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_plan::Plan;

use super::App;
use crate::command::NetworkCommand;
use crate::i18n;
use crate::refused::Refused;

impl App {
    pub(super) fn run_network(&mut self, ctx: &egui::Context, command: NetworkCommand) {
        match command {
            NetworkCommand::StartService(unit) => {
                if let Some(plan) = plan_start(unit) {
                    self.run_for_network(plan, unit.to_owned());
                }
            }
            NetworkCommand::LoadDriver { module, load } => {
                let plan = if load {
                    plan_load(&module)
                } else {
                    plan_unload(&module)
                };

                self.run_for_network(plan, module.to_string());
            }
            NetworkCommand::Ping {
                host,
                count,
                timeout,
            } => {
                let name = host.to_string();
                self.run_for_network(plan_ping(host, count, timeout), name);
            }
            NetworkCommand::Traceroute { host, hops } => {
                let name = host.to_string();
                self.run_for_network(plan_traceroute(host, hops), name);
            }
            NetworkCommand::OpenHotspot => self.open_hotspot(ctx),
            // The service takes these, see `hand_to_services`.
            NetworkCommand::Blacklist { module: _, on: _ }
            | NetworkCommand::ToggleHotspot
            | NetworkCommand::Refresh
            | NetworkCommand::SetRadio {
                target: _,
                blocked: _,
            }
            | NetworkCommand::LookupAddress
            | NetworkCommand::ForgetAddress => {}
        }
    }

    /// A run the page starts at once. One the Console refused stays on
    /// the page, naming `subject`.
    fn run_for_network(&mut self, plan: Plan, subject: String) {
        if self.run_at_once(plan).is_none() {
            let refused = Refused::new(Refusal::Busy, subject);
            self.services.network.set_refusal(Some(refused));
        }
    }

    /// The tool asks for root itself, as Maintenance's tools do.
    fn open_hotspot(&mut self, ctx: &egui::Context) {
        let Some(launcher) = &self.launcher else {
            return;
        };

        if let Err(error) = launcher.start(Path::new(HOTSPOT_PROGRAM), &[]) {
            tracing::error!("cannot start the hotspot tool: {error}");
            let reason = error.to_string();
            let text = i18n::text_with(ctx, "network-hotspot-not-started", &[("reason", &reason)]);
            self.toasts.add(ToastKind::Error, text);
        }
    }
}
