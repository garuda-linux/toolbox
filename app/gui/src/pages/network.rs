//! The Network page: the connection and the radios, the drivers, the
//! network hardware and two connection tests, one tab each.
//!
//! Radio switches and the address act at once, driver loads, service starts
//! and the tests run in the Console, the blacklist and the hotspot tool
//! wait in the queue.

mod drivers;
mod hardware;
mod status;
#[cfg(test)]
mod tests;
mod tools;

use egui::Ui;
use toolbox_core::features::network::DriverFilter;
use toolbox_gui_kit::widgets::{Tone, page_toolbar};

use super::header::{TabText, TabbedIds, search, tabbed_header_reading};
use super::rows::refused_banner;
use super::{PageContext, PageView, plan_running};
use crate::command::{AppCommand, NetworkCommand};
use crate::network::Network;
use crate::page::PageId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
enum Tab {
    #[default]
    Status,
    Drivers,
    Hardware,
    Tools,
}

impl TabText for Tab {
    fn text_id(self) -> &'static str {
        match self {
            Self::Status => "network-tab-status",
            Self::Drivers => "network-tab-drivers",
            Self::Hardware => "network-tab-hardware",
            Self::Tools => "network-tab-tools",
        }
    }
}

#[derive(Debug, Default)]
pub struct NetworkPage {
    tab: Tab,
    drivers: DriverFilter,
    tools: tools::Form,
    driver_rows: drivers::Rows,
}

impl PageView for NetworkPage {
    fn left(&mut self, ctx: &egui::Context) {
        drivers::forget(ctx);
    }

    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        let ids = TabbedIds {
            title: "page-network",
            subtitle: "page-network-subtitle",
            tabs: "network-tabs",
            refresh: "network-refresh",
        };

        let tabs = [Tab::Status, Tab::Drivers, Tab::Hardware, Tab::Tools];
        let reading = context.service::<Network>().reading();
        let done = tabbed_header_reading(ui, ids, &tabs, self.tab, reading);
        if let Some(picked) = done.picked {
            self.tab = picked;
            if context.service::<Network>().refusal().is_some() {
                context
                    .commands
                    .send(AppCommand::DismissRefusal(PageId::Network));
            }
        }

        if done.refresh {
            context.commands.send(NetworkCommand::Refresh);
        }

        if self.tab == Tab::Drivers {
            page_toolbar(
                ui,
                &mut self.drivers.search,
                |text, ui| {
                    _ = search(
                        ui,
                        text,
                        "network-drivers-search",
                        "network-drivers-search-hint",
                    );
                },
                |_, _| {},
            );
        }

        true
    }

    fn ui(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let _ui = tracing::trace_span!("network ui").entered();
        refusal(ui, context);
        plan_running(ui, context);
        match self.tab {
            Tab::Status => status::show(ui, context),
            Tab::Drivers => drivers::show(ui, context, &mut self.drivers, &mut self.driver_rows),
            Tab::Hardware => hardware::show(ui, context),
            Tab::Tools => self.tools.show(ui, context),
        }
    }

    fn place(&self) -> u64 {
        super::place_of(&self.tab)
    }
}

fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let refused = context.service::<Network>().refusal();
    if refused_banner(ui, refused, Tone::Error, "network-dismiss") {
        context
            .commands
            .send(AppCommand::DismissRefusal(PageId::Network));
    }
}
