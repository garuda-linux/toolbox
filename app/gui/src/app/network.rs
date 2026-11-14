//! Driver loads, service starts and the tests run at once in the Console. The
//! service queues the blacklist and the hotspot tool.

use std::path::Path;

use toolbox_core::features::network::{
    HOTSPOT_PROGRAM, plan_load, plan_ping, plan_start, plan_traceroute, plan_unload,
};
use toolbox_gui_kit::toasts::ToastKind;

use super::App;
use crate::command::NetworkCommand;
use crate::i18n;

impl App {
    pub(super) fn run_network(&mut self, ctx: &egui::Context, command: NetworkCommand) {
        match command {
            NetworkCommand::Refresh => {
                self.services.network.refresh();
                self.read_system();
                if let Some(units) = &self.units {
                    units.refresh(toolbox_plan::UnitScope::System);
                }
            }
            NetworkCommand::SetRadio { target, blocked } => {
                self.services.network.set_radio(target, blocked);
            }
            NetworkCommand::LookupAddress => self.services.network.lookup_address(),
            NetworkCommand::ForgetAddress => self.services.network.forget_address(),
            NetworkCommand::StartService(unit) => {
                if let Some(plan) = plan_start(unit) {
                    self.run_at_once(plan);
                }
            }
            NetworkCommand::LoadDriver { module, load } => {
                let plan = if load {
                    plan_load(&module)
                } else {
                    plan_unload(&module)
                };

                self.run_at_once(plan);
            }
            NetworkCommand::Ping {
                host,
                count,
                timeout,
            } => {
                self.run_at_once(plan_ping(host, count, timeout));
            }
            NetworkCommand::Traceroute { host, hops } => {
                self.run_at_once(plan_traceroute(host, hops));
            }
            NetworkCommand::OpenHotspot => self.open_hotspot(ctx),
            NetworkCommand::Blacklist { module: _, on: _ }
            | NetworkCommand::ToggleHotspot
            | NetworkCommand::DismissRefusal => {}
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
