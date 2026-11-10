//! Boot options: the variables of `/etc/default/grub` and the Plymouth
//! theme. Both go through the root helper, the first as one typed edit of
//! the file with one `grub-mkconfig` after it.

#[cfg(test)]
mod tests;

mod kernel;
mod loader;

use toolbox_plan::{
    DefaultEntry, GrubKey, GrubSetting, GrubSettings, PlannedAction, PlymouthTheme, RootAction,
};
pub use toolbox_state::boot::{BootLoader, CurrentGrub, KernelCmdline, PlymouthState};

pub use self::kernel::{KernelKey, KernelSetting, SdBootKernel, SdBootKernelDomain};
pub use self::loader::{CurrentLoader, LoaderSettingsDomain};
use super::each;
use crate::domain::{Domain, Single, Stage, Staged, Summary};
use crate::{ChangeSet, Snapshot, SummaryCode};

/// The variables of `/etc/default/grub`, each queued under its
/// [`GrubKey`] with the [`GrubSetting`] it should get as the wish.
pub struct GrubDefaults;

impl GrubDefaults {
    pub fn queued(changes: &ChangeSet, key: GrubKey) -> Option<GrubSetting> {
        changes.wish::<Self>(&key).cloned()
    }

    /// Queues `setting` under its own variable.
    pub fn want(changes: &mut ChangeSet, setting: GrubSetting, snapshot: &Snapshot) {
        changes.want::<Self>(setting.key(), setting, snapshot);
    }
}

/// After [`GrubDefaults`], so a failed GRUB step leaves the slow rebuild
/// of the initramfs undone.
pub struct PlymouthThemes;

impl Domain for GrubDefaults {
    type Item = GrubKey;
    type State = CurrentGrub;
    type Wish = GrubSetting;

    const ID: &'static str = "grub-settings";
    const STAGE: Stage = Stage::BOOT;
    const CODES: &'static [SummaryCode] = &[
        SummaryCode::GrubBackground,
        SummaryCode::GrubBackgroundNone,
        SummaryCode::GrubDefault,
        SummaryCode::GrubDefaultSaved,
        SummaryCode::GrubOsProberOff,
        SummaryCode::GrubOsProberOn,
        SummaryCode::GrubParameters,
        SummaryCode::GrubParametersNone,
        SummaryCode::GrubSaveDefaultOff,
        SummaryCode::GrubSaveDefaultOn,
        SummaryCode::GrubSubmenusOff,
        SummaryCode::GrubSubmenusOn,
        SummaryCode::GrubTheme,
        SummaryCode::GrubThemeNone,
        SummaryCode::GrubTimeout,
    ];

    fn fits(item: &GrubKey, wish: &GrubSetting) -> bool {
        wish.key() == *item
    }

    fn is_satisfied(_item: &GrubKey, wish: &GrubSetting, state: &CurrentGrub) -> bool {
        wish.is_met_by(&state.0)
    }

    /// One edit of the file for every queued variable, then one
    /// `grub-mkconfig`: it reads the whole file each time it runs.
    fn plan(changes: &[(&GrubKey, &GrubSetting)]) -> Vec<Staged> {
        let wishes = changes.iter().map(|(_, wish)| (*wish).clone()).collect();

        GrubSettings::new(wishes).map_or_else(
            |_nothing_queued| Vec::new(),
            |settings| {
                [
                    RootAction::SetGrubSettings { settings },
                    RootAction::GenerateGrubConfig,
                ]
                .map(|action| Staged::at(Self::STAGE, PlannedAction::Root(action)))
                .into()
            },
        )
    }

    fn key(item: &GrubKey) -> String {
        item.as_str().to_owned()
    }

    fn summary(_item: &GrubKey, wish: &GrubSetting) -> Summary {
        let switch = |on: bool, yes, no| Summary::plain(if on { yes } else { no });
        let named = |value: Option<String>, some, none| {
            value
                .filter(|text| !text.is_empty())
                .map_or_else(|| Summary::plain(none), |text| Summary::of(some, text))
        };

        match wish {
            GrubSetting::Default(DefaultEntry::Saved) => {
                Summary::plain(SummaryCode::GrubDefaultSaved)
            }
            GrubSetting::Default(DefaultEntry::Entry(entry)) => {
                Summary::of(SummaryCode::GrubDefault, entry.as_str())
            }
            GrubSetting::Timeout(seconds) => {
                Summary::of(SummaryCode::GrubTimeout, seconds.to_string())
            }
            GrubSetting::KernelParameters(_) => named(
                wish.value(),
                SummaryCode::GrubParameters,
                SummaryCode::GrubParametersNone,
            ),
            GrubSetting::Submenus(on) => switch(
                *on,
                SummaryCode::GrubSubmenusOn,
                SummaryCode::GrubSubmenusOff,
            ),
            GrubSetting::SaveDefault(on) => switch(
                *on,
                SummaryCode::GrubSaveDefaultOn,
                SummaryCode::GrubSaveDefaultOff,
            ),
            GrubSetting::OsProber(on) => switch(
                *on,
                SummaryCode::GrubOsProberOn,
                SummaryCode::GrubOsProberOff,
            ),
            GrubSetting::Background(_) => named(
                wish.value(),
                SummaryCode::GrubBackground,
                SummaryCode::GrubBackgroundNone,
            ),
            GrubSetting::Theme(_) => named(
                wish.value(),
                SummaryCode::GrubTheme,
                SummaryCode::GrubThemeNone,
            ),
        }
    }
}

impl Domain for PlymouthThemes {
    type Item = Single;
    type State = PlymouthState;
    type Wish = PlymouthTheme;

    const ID: &'static str = "plymouth-theme";
    const STAGE: Stage = Stage::new(Stage::BOOT.rank() + 10);
    const CODES: &'static [SummaryCode] = &[SummaryCode::PlymouthTheme];

    fn is_satisfied(_item: &Single, wish: &PlymouthTheme, state: &PlymouthState) -> bool {
        state.current.as_ref() == Some(wish)
    }

    fn plan(changes: &[(&Single, &PlymouthTheme)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |_, theme| {
            RootAction::SetPlymouthTheme {
                theme: theme.clone(),
            }
        })
    }

    single_key!();

    fn summary(_item: &Single, wish: &PlymouthTheme) -> Summary {
        Summary::of(SummaryCode::PlymouthTheme, wish.to_string())
    }
}
