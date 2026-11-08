//! One unit as its row shows it, built when the list changes, and the
//! actions of its menu.

use std::borrow::Cow;
use std::sync::Arc;

use egui::Ui;
use egui::text::LayoutJob;
use toolbox_core::features::services::{
    ActiveState, FileState, asks_first, critical, does_nothing,
};
use toolbox_gui_kit::fonts::{self, BODY, SMALL, Weight};
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::tokens;
use toolbox_gui_kit::widgets::{ListItem, Tone, highlighted};
use toolbox_plan::{UnitName, UnitOperation, UnitScope};

use super::ask::Pending;
use super::texts;
use super::view::Row;
use crate::command::ServiceCommand;
use crate::i18n;
use crate::pages::rows::actions::{CopyName, Named, RowAction, RowKind, Run, Section};

#[derive(Debug)]
pub struct UnitRow {
    pub unit: UnitName,
    pub scope: UnitScope,
    /// What screen readers say for the row: the name and the state.
    pub name: String,
    pub title: Arc<LayoutJob>,
    pub description: Arc<LayoutJob>,
    pub state: (Cow<'static, str>, Tone),
    /// The file state worth a tag, and what it means.
    pub file: Option<(Cow<'static, str>, Cow<'static, str>)>,
    pub queued: Vec<Cow<'static, str>>,
    /// Whether it starts at boot once the queue is applied.
    pub enabled: bool,
    pub masked: bool,
    pub active: ActiveState,
    /// A static or masked unit has no boot state to switch.
    pub switchable: bool,
    /// The name of the ⋯ button.
    pub menu: String,
}

impl UnitRow {
    pub fn of(ui: &Ui, row: &Row<'_>, scope: UnitScope, query: &str) -> Self {
        let ctx = ui.ctx();
        let tokens = tokens::current(ctx);
        let unit = row.unit;
        let strong = fonts::font(ui.style(), Weight::SemiBold, BODY);
        let title = highlighted(ui, unit.name.as_str(), query, strong, tokens.text);
        let small = fonts::font(ui.style(), Weight::Regular, SMALL);
        let description = highlighted(ui, &unit.description, query, small, tokens.secondary);
        let state = texts::state(ctx, unit);

        Self {
            unit: unit.name.clone(),
            scope,
            name: format!("{}, {}", unit.name.as_str(), state.0),
            title: Arc::new(title),
            description: Arc::new(description),
            state,
            file: texts::file(ctx, unit.file_state),
            queued: texts::queued(ctx, row.enable, row.mask),
            enabled: row.enabled,
            masked: row.masked,
            active: unit.active,
            switchable: !row.masked && unit.file_state != FileState::Static,
            menu: i18n::text_with(ctx, "services-actions", &[("name", unit.name.as_str())])
                .into_owned(),
        }
    }
}

/// The menu of a unit row: what runs at once, then what waits.
pub struct UnitRows;

impl RowKind for UnitRows {
    type Item = UnitRow;

    const SECTIONS: &'static [Section<UnitRow>] = &[
        Section {
            heading: Some("services-now"),
            actions: &[
                &Control(UnitOperation::Start),
                &Control(UnitOperation::Stop),
                &Control(UnitOperation::Restart),
                &Control(UnitOperation::Reload),
                &ShowLogs,
                &CopyName,
            ],
        },
        Section {
            heading: Some("services-with-apply"),
            actions: &[&Mask],
        },
    ];
}

/// Starts, stops, restarts or reloads the unit at once.
struct Control(UnitOperation);

/// Queues masking the unit, or unmasking a masked one.
struct Mask;

struct ShowLogs;

impl RowAction<UnitRow> for Control {
    fn label(&self, _item: &UnitRow) -> &'static str {
        texts::operation(self.0)
    }

    fn icon(&self, _item: &UnitRow) -> Option<Icon> {
        Some(match self.0 {
            UnitOperation::Start => icons::MEDIA_PLAYBACK_START,
            UnitOperation::Stop => icons::MEDIA_PLAYBACK_STOP,
            UnitOperation::Restart => icons::MEDIA_PLAYLIST_REPEAT,
            // A reload has the unit read its files again, the meaning of
            // Read again.
            UnitOperation::Reload => icons::VIEW_REFRESH,
        })
    }

    fn possible(&self, item: &UnitRow) -> Result<(), &'static str> {
        match self.0 {
            UnitOperation::Start | UnitOperation::Restart if item.masked => {
                Err("services-why-masked")
            }
            UnitOperation::Reload if item.active != ActiveState::Active => {
                Err("services-why-not-running")
            }
            UnitOperation::Start if does_nothing(self.0, item.active) => {
                Err("services-why-running")
            }
            UnitOperation::Stop if does_nothing(self.0, item.active) => Err("services-why-stopped"),
            UnitOperation::Start
            | UnitOperation::Stop
            | UnitOperation::Restart
            | UnitOperation::Reload => Ok(()),
        }
    }

    fn run(&self, item: &UnitRow, run: Run<'_>) {
        if asks_first(self.0, &item.unit) {
            let pending = Pending::Control(item.scope, item.unit.clone(), self.0);
            super::ask::hold(run.ctx, pending);
            return;
        }

        run.commands.send(ServiceCommand::Control {
            scope: item.scope,
            unit: item.unit.clone(),
            operation: self.0,
        });
    }
}

impl RowAction<UnitRow> for Mask {
    fn label(&self, item: &UnitRow) -> &'static str {
        if item.masked {
            "services-unmask"
        } else {
            "services-mask"
        }
    }

    fn icon(&self, item: &UnitRow) -> Option<Icon> {
        Some(if item.masked {
            icons::OBJECT_UNLOCKED
        } else {
            icons::OBJECT_LOCKED
        })
    }

    fn run(&self, item: &UnitRow, run: Run<'_>) {
        if !item.masked && critical(&item.unit) {
            super::ask::hold(run.ctx, Pending::Mask(item.scope, item.unit.clone()));
            return;
        }

        run.commands.send(ServiceCommand::Mask {
            scope: item.scope,
            unit: item.unit.clone(),
            on: !item.masked,
        });
    }
}

impl RowAction<UnitRow> for ShowLogs {
    fn label(&self, _item: &UnitRow) -> &'static str {
        "row-show-logs"
    }

    fn icon(&self, _item: &UnitRow) -> Option<Icon> {
        Some(icons::UTILITIES_TERMINAL)
    }

    fn run(&self, item: &UnitRow, run: Run<'_>) {
        run.commands.send(ServiceCommand::Logs {
            scope: item.scope,
            unit: item.unit.clone(),
        });
    }
}

impl ListItem for UnitRow {
    fn key(&self) -> &str {
        self.unit.as_str()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for UnitRow {
    fn copied_name(&self) -> &str {
        self.unit.as_str()
    }
}
