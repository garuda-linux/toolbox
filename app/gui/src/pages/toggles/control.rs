//! The control at the right of a catalog row.
//!
//! A page that mixes settings and packages shows a button on every row:
//! Enable or Disable for a setting or a service, Install, Remove or Take
//! back for a package. System Tweaks holds settings alone and keeps its
//! switches. Discover's and Kernels' rows use it too.

use egui::Ui;
use egui::text::{LayoutJob, TextWrapping};
use toolbox_core::Toggle;
use toolbox_core::features::toggles::shown::{Control, Entry};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::widgets::{
    ButtonKind, RowState, icon_button_width, pill_button_at_least, pill_width, switch,
};

use crate::catalogs::CatalogId;
use crate::i18n;
use crate::pages::rows::SWITCH_WIDTH;

/// How many lines of its description a row shows before it cuts.
pub const DESCRIPTION_LINES: u8 = 2;
const BUTTONS: [&str; 5] = [
    "packages-install",
    "packages-remove",
    "packages-take-back",
    "toggles-enable",
    "toggles-disable",
];

#[derive(Clone, Copy, Debug)]
pub struct Shown<'row> {
    pub control: Control,
    pub on: bool,
    pub title: &'row str,
    pub enabled: bool,
    /// Neither the system nor the queue says whether it is on.
    pub unknown: bool,
    pub setting: Setting,
}

/// How a row shows a setting. A page that mixes settings and packages
/// gives a setting a button too, so every row looks alike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Switch,
    Button,
}

/// The control of `entry` on `catalog`'s page. System Tweaks' entries are
/// settings even where a package makes them, so they keep their switch.
pub const fn of(catalog: CatalogId, entry: &Entry<'_>) -> Control {
    match catalog {
        CatalogId::Settings => Control::Switch,
        CatalogId::Components | CatalogId::Gaming | CatalogId::Setup => entry.control(),
    }
}

/// The id of the text beside the title of an entry whose `wish` waits:
/// a package installs or removes, a setting turns on or off.
pub const fn queued_text(wish: Toggle, control: Control) -> &'static str {
    match (control, wish) {
        (Control::Switch, Toggle::On) => "toggles-turns-on",
        (Control::Switch, Toggle::Off) => "toggles-turns-off",
        (Control::Install | Control::Remove | Control::TakeBack, Toggle::On) => "packages-installs",
        (Control::Install | Control::Remove | Control::TakeBack, Toggle::Off) => "packages-removes",
    }
}

/// Cuts a description after [`DESCRIPTION_LINES`]. Its tooltip and
/// screen readers keep it whole.
pub fn cut(mut description: LayoutJob) -> LayoutJob {
    description.wrap = TextWrapping {
        max_rows: usize::from(DESCRIPTION_LINES),
        ..TextWrapping::default()
    };

    description
}

/// How wide the row's menu button and its control are together. Every
/// button of a page is as wide as the widest text, so the descriptions
/// of all rows end at one x.
pub fn width(ui: &Ui, control: Control, setting: Setting) -> f32 {
    let menu = icon_button_width(ui) + ui.tokens().sizes.control_gap;
    match control {
        Control::Switch if setting == Setting::Switch => menu + SWITCH_WIDTH,
        Control::Switch | Control::Install | Control::Remove | Control::TakeBack => {
            menu + button_width(ui)
        }
    }
}

/// Returns whether the row asks for its change: a click on it, or Space on
/// its focused row. A pointer click on the empty part of a row flips a
/// switch but presses no button.
pub fn show(ui: &mut Ui, shown: Shown<'_>, state: &RowState) -> bool {
    let response = match shown.control {
        Control::Switch if shown.setting == Setting::Switch => {
            switch(ui, shown.on, shown.title, shown.enabled)
        }
        Control::Switch | Control::Install | Control::Remove | Control::TakeBack => {
            button(ui, shown)
        }
    };

    // The page banner says it once to the eye, a screen reader hears it
    // on the row it reads.
    if shown.unknown {
        let unknown = i18n::text(ui.ctx(), "toggles-state-unknown").into_owned();
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_description(unknown));
    }

    let asked = match shown.control {
        Control::Switch if shown.setting == Setting::Switch => state.toggle,
        Control::Switch | Control::Install | Control::Remove | Control::TakeBack => {
            state.toggle && !state.response.clicked()
        }
    };

    response.clicked() || (asked && shown.enabled)
}

fn button(ui: &mut Ui, shown: Shown<'_>) -> egui::Response {
    let id = match shown.control {
        Control::Remove => "packages-remove",
        Control::TakeBack => "packages-take-back",
        Control::Install => "packages-install",
        Control::Switch if shown.on => "toggles-disable",
        Control::Switch => "toggles-enable",
    };

    let ctx = ui.ctx().clone();
    let text = i18n::text(&ctx, id);
    let name = i18n::text_with(
        &ctx,
        "packages-action-name",
        &[("action", &*text), ("name", shown.title)],
    )
    .into_owned();

    let width = button_width(ui);
    let response = ui
        .add_enabled_ui(shown.enabled, |ui| {
            pill_button_at_least(ui, &text, ButtonKind::Soft, width)
        })
        .inner;

    ctx.accesskit_node_builder(response.id, |node| node.set_label(name));
    response
}

/// The width of the widest button text, which every row button takes.
pub fn button_width(ui: &Ui) -> f32 {
    BUTTONS
        .iter()
        .map(|id| pill_width(ui, &i18n::text(ui.ctx(), id)))
        .fold(0.0_f32, f32::max)
}
