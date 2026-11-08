//! The Network page: the connection and the radios, the drivers, the
//! network hardware and two connection tests, one tab each.
//!
//! The page reads the machine itself once it shows. Radio switches and
//! the address request act at once, driver loads, service starts and the
//! tests run in the Console, the blacklist and the hotspot tool wait in
//! the queue.

mod drivers;
mod hardware;
mod status;
#[cfg(test)]
mod tests;
mod tools;

use egui::Ui;
use toolbox_core::features::network::DriverFilter;
use toolbox_gui_kit::widgets::{Tone, page_toolbar};

use super::header::{TabText, TabbedIds, search, tabbed_header};
use super::rows::refusal_banner;
use super::{PageContext, PageView, plan_running};
use crate::command::NetworkCommand;
use crate::i18n;

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
}

impl PageView for NetworkPage {
    fn header(&mut self, ui: &mut Ui, context: &PageContext<'_>) -> bool {
        let ids = TabbedIds {
            title: "page-network",
            subtitle: "page-network-subtitle",
            tabs: "network-tabs",
            refresh: "network-refresh",
        };

        let tabs = [Tab::Status, Tab::Drivers, Tab::Hardware, Tab::Tools];
        let done = tabbed_header(ui, ids, &tabs, self.tab);
        if let Some(picked) = done.picked {
            self.tab = picked;
            if context.refusals.network.is_some() {
                context.commands.send(NetworkCommand::DismissRefusal);
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
            Tab::Drivers => drivers::show(ui, context, &mut self.drivers),
            Tab::Hardware => hardware::show(ui, context),
            Tab::Tools => self.tools.show(ui, context),
        }
    }

    fn place(&self) -> u64 {
        super::place_of(&self.tab)
    }
}

fn refusal(ui: &mut Ui, context: &PageContext<'_>) {
    let Some(refusal) = &context.refusals.network else {
        return;
    };

    let id = format!("network-refused-{}", refusal.code);
    let text = i18n::text_with(ui.ctx(), &id, &[("driver", &refusal.driver)]);
    if refusal_banner(ui, Tone::Error, &text, "network-dismiss") {
        context.commands.send(NetworkCommand::DismissRefusal);
    }
}
