//! The Tools tab: ping and traceroute in the Console, and the hotspot
//! tool.

use egui::{Key, Label, TextEdit, Ui};
use toolbox_core::features::network::{
    HOTSPOT_PACKAGE, PackageState, TRACEROUTE_PACKAGE, package_state,
};
use toolbox_gui_kit::fonts::{SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, SectionHeading, Tone, banner, card, number_field, pill_button, rich, text_field,
};
use toolbox_gui_kit::{UiExt as _, icons};
use toolbox_plan::{
    DEFAULT_HOPS, DEFAULT_PING_TIMEOUT, DEFAULT_PINGS, DEFAULT_TEST_HOST, HopLimit, HostName,
    PingCount, PingTimeout,
};

use crate::command::NetworkCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::setting_row::{Words, row};

const FIELD_WIDTH: f32 = 240.0;
const NUMBER_WIDTH: f32 = 120.0;
/// The number field itself, inside its row's control column.
const NUMBER_FIELD_WIDTH: f32 = 72.0;

/// What the tests run with, kept while the page is away.
#[derive(Debug)]
pub struct Form {
    host: String,
    pings: u32,
    timeout: u32,
    hops: u32,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            host: DEFAULT_TEST_HOST.to_owned(),
            pings: DEFAULT_PINGS,
            timeout: DEFAULT_PING_TIMEOUT,
            hops: DEFAULT_HOPS,
        }
    }
}

impl Form {
    pub fn show(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        self.tests(ui, context);
        ui.add_space(ui.tokens().sizes.section_gap);
        hotspot(ui, context);
    }

    fn tests(&mut self, ui: &mut Ui, context: &PageContext<'_>) {
        let ctx = ui.ctx().clone();
        let text = |id: &str| i18n::text(&ctx, id).into_owned();
        let title = text("network-tests");
        let detail = text("network-tests-detail");
        heading(ui, &title, &detail);

        let host = HostName::new(self.host.trim()).ok();
        let entered = self.fields(ui, host.is_some());
        ui.add_space(ui.tokens().sizes.card_gap);
        let traceroute = package_state(&TRACEROUTE_PACKAGE, context.snapshot, context.changes);
        let missing = matches!(traceroute, PackageState::Missing | PackageState::Queued);
        let ready = host.is_some() && !context.busy;
        let (ping, trace) = buttons(ui, ready, missing);
        if missing {
            ui.add_space(ui.tokens().sizes.card_gap);
            banner(ui, Tone::Neutral, &text("network-traceroute-missing"), &[]);
        }

        let Some(host) = host.filter(|_| ready) else {
            return;
        };

        match (ping || entered, trace) {
            (true, _) => self.ping(context, host),
            (false, true) => self.traceroute(context, host),
            (false, false) => {}
        }
    }

    /// The host and the numbers. Returns whether Enter was pressed in the
    /// host field.
    fn fields(&mut self, ui: &mut Ui, host_ok: bool) -> bool {
        let ctx = ui.ctx().clone();
        let text = |id: &str| i18n::text(&ctx, id).into_owned();
        let mut entered = false;
        card(ui, |ui| {
            let problem = if host_ok {
                String::new()
            } else {
                text("network-host-problem")
            };

            // A host that cannot be used is a problem, not a quiet hint.
            let name = text("network-host");
            let words = Words {
                current: None,
                title: &name,
                description: "",
                note: (!host_ok).then_some(problem.as_str()),
                queued: None,
            };

            row(ui, true, &words, FIELD_WIDTH, |ui| {
                let field = text_field(
                    ui,
                    TextEdit::singleline(&mut self.host)
                        .desired_width(FIELD_WIDTH)
                        .align(egui::Align2::LEFT_CENTER),
                );

                ui.ctx()
                    .accesskit_node_builder(field.id, |node| node.set_label(name.clone()));

                entered = field.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
            });

            number(ui, &text("network-pings"), &mut self.pings, PingCount::MAX);
            number(
                ui,
                &text("network-timeout"),
                &mut self.timeout,
                PingTimeout::MAX,
            );
            number(ui, &text("network-hops"), &mut self.hops, HopLimit::MAX);
        });

        entered
    }

    fn ping(&self, context: &PageContext<'_>, host: HostName) {
        let (Ok(count), Ok(timeout)) = (
            PingCount::new(i64::from(self.pings)),
            PingTimeout::new(i64::from(self.timeout)),
        ) else {
            return;
        };

        context.commands.send(NetworkCommand::Ping {
            host,
            count,
            timeout,
        });
    }

    fn traceroute(&self, context: &PageContext<'_>, host: HostName) {
        if let Ok(hops) = HopLimit::new(i64::from(self.hops)) {
            context
                .commands
                .send(NetworkCommand::Traceroute { host, hops });
        }
    }
}

/// Run ping and Run traceroute. Returns which was clicked.
fn buttons(ui: &mut Ui, ready: bool, missing: bool) -> (bool, bool) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    ui.horizontal(|ui| {
        let ping = ui
            .add_enabled_ui(ready, |ui| {
                pill_button(ui, &text("network-ping"), ButtonKind::Soft)
            })
            .inner
            .clicked();

        let trace = ui
            .add_enabled_ui(ready && !missing, |ui| {
                pill_button(ui, &text("network-traceroute"), ButtonKind::Soft)
            })
            .inner
            .clicked();

        (ping, trace)
    })
    .inner
}

/// A whole number from 1 to `max`.
fn number(ui: &mut Ui, title: &str, value: &mut u32, max: u32) {
    let words = Words {
        current: None,
        title,
        description: "",
        note: None,
        queued: None,
    };

    row(ui, false, &words, NUMBER_WIDTH, |ui| {
        // One width for every number, whatever its digits.
        number_field(ui, value, 1..=max, NUMBER_FIELD_WIDTH, title);
    });
}

fn hotspot(ui: &mut Ui, context: &PageContext<'_>) {
    let ctx = ui.ctx().clone();
    let text = |id: &str| i18n::text(&ctx, id).into_owned();
    let state = package_state(&HOTSPOT_PACKAGE, context.snapshot, context.changes);
    let detail = if state == PackageState::Installed {
        text("network-hotspot-detail")
    } else {
        text("network-hotspot-queued-detail")
    };

    let title = text("network-hotspot");
    heading(ui, &title, &detail);

    let button = match state {
        PackageState::Installed => "network-hotspot-open",
        PackageState::Queued => "network-hotspot-unqueue",
        PackageState::Missing | PackageState::Unknown => "network-hotspot-queue",
    };

    let clicked = ui
        .add_enabled_ui(state != PackageState::Unknown, |ui| {
            pill_button(ui, &text(button), ButtonKind::Soft)
        })
        .inner
        .clicked();

    if clicked {
        let command = if state == PackageState::Installed {
            NetworkCommand::OpenHotspot
        } else {
            NetworkCommand::ToggleHotspot
        };

        context.commands.send(command);
    }

    if state == PackageState::Unknown {
        ui.add_space(ui.tokens().sizes.card_gap);
        banner(ui, Tone::Warning, &text("network-hotspot-unknown"), &[]);
    }
}

/// A section heading with its detail wrapped below, as it can be long.
fn heading(ui: &mut Ui, title: &str, detail: &str) {
    SectionHeading::new(icons::NETWORK_CONNECT, title, "").show(ui);
    let secondary = ui.tokens().secondary;
    ui.add(Label::new(rich(ui, detail, Weight::Regular, SMALL, secondary)).wrap());
    ui.add_space(ui.tokens().sizes.heading_gap);
}
