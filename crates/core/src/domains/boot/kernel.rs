#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use toolbox_plan::{KernelParameters, PlannedAction, PlymouthTheme, RootAction};
pub use toolbox_state::boot::SdBootKernel;

use crate::domain::{Domain, Stage, Staged, Summary};
use crate::{ChangeSet, Snapshot, SummaryCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KernelKey {
    Parameters,
    PlymouthTheme,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "key", content = "value", rename_all = "kebab-case")]
pub enum KernelSetting {
    Parameters(KernelParameters),
    PlymouthTheme(PlymouthTheme),
}

/// What systemd-boot builds into its UKIs. One domain for the command
/// line and the Plymouth theme, so both share one rebuild at the end.
pub struct SdBootKernelDomain;

impl KernelSetting {
    pub const fn key(&self) -> KernelKey {
        match self {
            Self::Parameters(_) => KernelKey::Parameters,
            Self::PlymouthTheme(_) => KernelKey::PlymouthTheme,
        }
    }
}

impl SdBootKernelDomain {
    pub fn want(changes: &mut ChangeSet, setting: KernelSetting, snapshot: &Snapshot) {
        changes.want::<Self>(setting.key(), setting, snapshot);
    }

    pub fn queued_parameters(changes: &ChangeSet) -> Option<KernelParameters> {
        match changes.wish::<Self>(&KernelKey::Parameters) {
            Some(KernelSetting::Parameters(list)) => Some(list.clone()),
            Some(KernelSetting::PlymouthTheme(_)) | None => None,
        }
    }

    pub fn queued_theme(changes: &ChangeSet) -> Option<PlymouthTheme> {
        match changes.wish::<Self>(&KernelKey::PlymouthTheme) {
            Some(KernelSetting::PlymouthTheme(theme)) => Some(theme.clone()),
            Some(KernelSetting::Parameters(_)) | None => None,
        }
    }
}

impl Domain for SdBootKernelDomain {
    type Item = KernelKey;
    type State = SdBootKernel;
    type Wish = KernelSetting;

    const ID: &'static str = "sdboot-kernel";
    /// After the loader settings: a failed one leaves the slow rebuild
    /// undone.
    const STAGE: Stage = Stage::new(Stage::BOOT.rank() + 11);
    const CODES: &'static [SummaryCode] = &[
        SummaryCode::SdbootParameters,
        SummaryCode::SdbootParametersNone,
        SummaryCode::SdbootPlymouthTheme,
    ];

    fn fits(item: &KernelKey, wish: &KernelSetting) -> bool {
        wish.key() == *item
    }

    fn is_satisfied(_item: &KernelKey, wish: &KernelSetting, state: &SdBootKernel) -> bool {
        match wish {
            KernelSetting::Parameters(list) => state.cmdline.parameters.as_ref() == Some(list),
            KernelSetting::PlymouthTheme(theme) => state.theme.as_ref() == Some(theme),
        }
    }

    fn plan(changes: &[(&KernelKey, &KernelSetting)]) -> Vec<Staged> {
        let mut actions: Vec<RootAction> = changes
            .iter()
            .map(|(_, wish)| match wish {
                KernelSetting::Parameters(parameters) => RootAction::SetKernelCmdline {
                    parameters: parameters.clone(),
                },
                KernelSetting::PlymouthTheme(theme) => RootAction::SelectPlymouthTheme {
                    theme: theme.clone(),
                },
            })
            .collect();

        if !actions.is_empty() {
            let initramfs = changes
                .iter()
                .any(|(item, _)| **item == KernelKey::PlymouthTheme);

            actions.push(RootAction::RebuildUkis { initramfs });
        }

        actions
            .into_iter()
            .map(|action| Staged::at(Self::STAGE, PlannedAction::Root(action)))
            .collect()
    }

    fn key(item: &KernelKey) -> String {
        match item {
            KernelKey::Parameters => "parameters",
            KernelKey::PlymouthTheme => "plymouth-theme",
        }
        .to_owned()
    }

    fn summary(_item: &KernelKey, wish: &KernelSetting) -> Summary {
        match wish {
            KernelSetting::Parameters(list) if list.iter().next().is_none() => {
                Summary::plain(SummaryCode::SdbootParametersNone)
            }
            KernelSetting::Parameters(list) => {
                Summary::of(SummaryCode::SdbootParameters, list.to_string())
            }
            KernelSetting::PlymouthTheme(theme) => {
                Summary::of(SummaryCode::SdbootPlymouthTheme, theme.as_str())
            }
        }
    }
}
