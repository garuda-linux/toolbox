#[cfg(test)]
mod tests;

use toolbox_plan::sdboot::{LoaderChange, LoaderKey, LoaderSetting, LoaderTimeout};
use toolbox_plan::{PlannedAction, RootAction};
pub use toolbox_state::boot::CurrentLoader;

use crate::domain::{Domain, Stage, Staged, Summary};
use crate::{ChangeSet, Snapshot, SummaryCode};

pub struct LoaderSettingsDomain;

impl LoaderSettingsDomain {
    pub fn queued(changes: &ChangeSet, key: LoaderKey) -> Option<LoaderSetting> {
        changes.wish::<Self>(&key).cloned()
    }

    pub fn want(changes: &mut ChangeSet, setting: LoaderSetting, snapshot: &Snapshot) {
        changes.want::<Self>(setting.loader_key(), setting, snapshot);
    }
}

impl Domain for LoaderSettingsDomain {
    type Item = LoaderKey;
    type State = CurrentLoader;
    type Wish = LoaderSetting;

    const ID: &'static str = "loader-settings";
    const STAGE: Stage = Stage::BOOT;
    const CODES: &'static [SummaryCode] = &[
        SummaryCode::LoaderAutoEntriesOff,
        SummaryCode::LoaderAutoEntriesOn,
        SummaryCode::LoaderAutoEntriesUnset,
        SummaryCode::LoaderAutoFirmwareOff,
        SummaryCode::LoaderAutoFirmwareOn,
        SummaryCode::LoaderAutoFirmwareUnset,
        SummaryCode::LoaderDefault,
        SummaryCode::LoaderDefaultSaved,
        SummaryCode::LoaderDefaultUnset,
        SummaryCode::LoaderEditorOff,
        SummaryCode::LoaderEditorOn,
        SummaryCode::LoaderEditorUnset,
        SummaryCode::LoaderTimeout,
        SummaryCode::LoaderTimeoutMenu,
        SummaryCode::LoaderTimeoutUnset,
    ];

    fn fits(item: &LoaderKey, wish: &LoaderSetting) -> bool {
        wish.loader_key() == *item
    }

    fn is_satisfied(_item: &LoaderKey, wish: &LoaderSetting, state: &CurrentLoader) -> bool {
        wish.is_met_by(&state.0)
    }

    /// systemd-boot reads the file at each start, so nothing is built
    /// afterwards.
    fn plan(changes: &[(&LoaderKey, &LoaderSetting)]) -> Vec<Staged> {
        let wishes = changes.iter().map(|(_, wish)| (*wish).clone()).collect();

        LoaderChange::new(wishes).map_or_else(
            |_nothing_queued| Vec::new(),
            |settings| {
                vec![Staged::at(
                    Self::STAGE,
                    PlannedAction::Root(RootAction::SetLoaderSettings { settings }),
                )]
            },
        )
    }

    fn key(item: &LoaderKey) -> String {
        item.as_str().to_owned()
    }

    fn summary(_item: &LoaderKey, wish: &LoaderSetting) -> Summary {
        let switch = |on: Option<bool>, [yes, no, unset]: [SummaryCode; 3]| {
            Summary::plain(match on {
                Some(true) => yes,
                Some(false) => no,
                None => unset,
            })
        };

        match wish {
            LoaderSetting::Timeout(Some(LoaderTimeout::Seconds(seconds))) => {
                Summary::of(SummaryCode::LoaderTimeout, seconds.to_string())
            }
            LoaderSetting::Timeout(Some(word)) => {
                Summary::of(SummaryCode::LoaderTimeoutMenu, word.to_string())
            }
            LoaderSetting::Timeout(None) => Summary::plain(SummaryCode::LoaderTimeoutUnset),
            LoaderSetting::Default(Some(default)) if default.as_str() == "@saved" => {
                Summary::plain(SummaryCode::LoaderDefaultSaved)
            }
            LoaderSetting::Default(Some(default)) => {
                Summary::of(SummaryCode::LoaderDefault, default.as_str())
            }
            LoaderSetting::Default(None) => Summary::plain(SummaryCode::LoaderDefaultUnset),
            LoaderSetting::Editor(on) => switch(
                *on,
                [
                    SummaryCode::LoaderEditorOn,
                    SummaryCode::LoaderEditorOff,
                    SummaryCode::LoaderEditorUnset,
                ],
            ),
            LoaderSetting::AutoEntries(on) => switch(
                *on,
                [
                    SummaryCode::LoaderAutoEntriesOn,
                    SummaryCode::LoaderAutoEntriesOff,
                    SummaryCode::LoaderAutoEntriesUnset,
                ],
            ),
            LoaderSetting::AutoFirmware(on) => switch(
                *on,
                [
                    SummaryCode::LoaderAutoFirmwareOn,
                    SummaryCode::LoaderAutoFirmwareOff,
                    SummaryCode::LoaderAutoFirmwareUnset,
                ],
            ),
        }
    }
}
