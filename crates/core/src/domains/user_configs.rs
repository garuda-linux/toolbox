//! Edits of config files in the user's home (System Settings' "Desktop and
//! apps"). The app writes them as the user, then restarts the user units
//! that read them, once for all edits of an apply.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use toolbox_plan::{
    ConfigReload, PlannedAction, TargetList, UnitName, UnitOperation, UserAction, UserConfigEdit,
};
pub use toolbox_state::parts::UserConfigStates;

use crate::SummaryCode;
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};

/// Whether the edit should be in place, and what must read it again.
///
/// The units to restart or reload follow from the catalog, so they
/// belong to the wish, not to the edit that identifies the change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfigWish {
    pub toggle: Toggle,
    pub restart: BTreeSet<UnitName>,
    pub reload: BTreeSet<ConfigReload>,
}

pub struct UserConfigs;

impl UserConfigWish {
    pub fn new<I>(toggle: Toggle, restart: I) -> Self
    where
        I: IntoIterator<Item = UnitName>,
    {
        Self {
            toggle,
            restart: restart.into_iter().collect(),
            reload: BTreeSet::new(),
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn with_reload(mut self, reload: Option<ConfigReload>) -> Self {
        self.reload.extend(reload);
        self
    }
}

impl Domain for UserConfigs {
    type Item = UserConfigEdit;
    type State = UserConfigStates;
    type Wish = UserConfigWish;

    const ID: &'static str = "user-configs";
    const STAGE: Stage = Stage::USER_SESSION;

    fn is_satisfied(
        item: &UserConfigEdit,
        wish: &UserConfigWish,
        state: &UserConfigStates,
    ) -> bool {
        state.0.get(item) == Some(&wish.toggle.is_on())
    }

    /// A restart in either direction: the program must read the file again
    /// to drop a removed setting too.
    fn plan(changes: &[(&UserConfigEdit, &UserConfigWish)]) -> Vec<Staged> {
        let restart = changes
            .iter()
            .flat_map(|(_, wish)| wish.restart.iter().cloned());

        let reloads: BTreeSet<ConfigReload> = changes
            .iter()
            .flat_map(|(_, wish)| wish.reload.iter().copied())
            .collect();

        let restart = TargetList::new(restart)
            .ok()
            .map(|units| UserAction::ControlUserUnits {
                operation: UnitOperation::Restart,
                units,
            });

        changes
            .iter()
            .map(|(edit, wish)| UserAction::EditUserConfig {
                edit: (*edit).clone(),
                on: wish.toggle.is_on(),
            })
            .chain(restart)
            .chain(
                reloads
                    .into_iter()
                    .map(|target| UserAction::ReloadConfig { target }),
            )
            .map(PlannedAction::User)
            .map(|action| Staged::at(Self::STAGE, action))
            .collect()
    }

    named_key!();

    fn summary(item: &UserConfigEdit, wish: &UserConfigWish) -> Summary {
        Summary::toggle(
            wish.toggle,
            SummaryCode::UserConfigOn,
            SummaryCode::UserConfigOff,
            item.to_string(),
        )
    }
}
