#[cfg(test)]
mod tests;

use toolbox_core::domains::{SystemUnitMasks, SystemUnits, UserUnitMasks, UserUnits};
use toolbox_core::features::services::{FileState, Filter, UnitInfo, UnitOrder, pick};
use toolbox_core::{ChangeSet, Toggle};
use toolbox_plan::UnitScope;

#[derive(Debug)]
pub struct Row<'units> {
    pub unit: &'units UnitInfo,
    pub enable: Option<Toggle>,
    pub mask: Option<Toggle>,
    /// What the unit will be once the queue is applied.
    pub enabled: bool,
    pub masked: bool,
}

pub fn rows<'units>(
    units: &'units [UnitInfo],
    filter: &Filter,
    order: UnitOrder,
    scope: UnitScope,
    changes: &ChangeSet,
) -> Vec<Row<'units>> {
    pick(units, filter, order)
        .into_iter()
        .filter_map(|at| units.get(at))
        .map(|unit| {
            let (enable, mask) = match scope {
                UnitScope::System => (
                    changes.wish::<SystemUnits>(&unit.name),
                    changes.wish::<SystemUnitMasks>(&unit.name),
                ),
                UnitScope::User => (
                    changes.wish::<UserUnits>(&unit.name),
                    changes.wish::<UserUnitMasks>(&unit.name),
                ),
            };

            Row {
                unit,
                enable: enable.copied(),
                mask: mask.copied(),
                enabled: enable.map_or(unit.file_state == FileState::Enabled, |wish| wish.is_on()),
                masked: mask.map_or(unit.file_state == FileState::Masked, |wish| wish.is_on()),
            }
        })
        .collect()
}
