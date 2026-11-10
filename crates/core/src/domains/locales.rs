//! Locales: which lines of `/etc/locale.gen` are switched on, and the
//! system language in `/etc/locale.conf`. Both go through the root helper
//! as typed file edits, so one password covers them with everything else.

#[cfg(test)]
mod tests;

use toolbox_plan::{LocaleEntry, LocaleName, PlannedAction, RootAction, same_locale};
pub use toolbox_state::locales::{CurrentSystemLocale, EnabledLocales};

use super::each;
use crate::SummaryCode;
use crate::domain::{Domain, Single, Stage, Staged, Summary, Toggle};

pub struct Locales;

/// After [`Locales`]: the helper refuses a language whose locale is not
/// enabled, and an apply may enable it first.
pub struct SystemLocales;

impl Domain for Locales {
    type Item = LocaleEntry;
    type State = EnabledLocales;
    type Wish = Toggle;

    const ID: &'static str = "locales";
    const STAGE: Stage = Stage::LOCALES;
    const CODES: &'static [SummaryCode] = &[SummaryCode::LocaleDisable, SummaryCode::LocaleEnable];

    fn is_satisfied(item: &LocaleEntry, wish: &Toggle, state: &EnabledLocales) -> bool {
        state.0.contains(item) == wish.is_on()
    }

    /// One `locale-gen` after all edits: it rebuilds every enabled locale
    /// each time it runs.
    fn plan(changes: &[(&LocaleEntry, &Toggle)]) -> Vec<Staged> {
        changes
            .iter()
            .map(|(entry, wish)| RootAction::SetLocaleEnabled {
                locale: (*entry).clone(),
                enabled: wish.is_on(),
            })
            .chain([RootAction::GenerateLocales])
            .map(PlannedAction::Root)
            .map(|action| Staged::at(Self::STAGE, action))
            .collect()
    }

    named_key!();

    fn summary(item: &LocaleEntry, wish: &Toggle) -> Summary {
        Summary::toggle(
            *wish,
            SummaryCode::LocaleEnable,
            SummaryCode::LocaleDisable,
            item.to_string(),
        )
    }
}

impl Domain for SystemLocales {
    type Item = Single;
    type State = CurrentSystemLocale;
    type Wish = LocaleName;

    const ID: &'static str = "system-locale";
    const STAGE: Stage = Stage::new(Stage::LOCALES.rank() + 10);
    const CODES: &'static [SummaryCode] = &[SummaryCode::SystemLocale];

    fn is_satisfied(_item: &Single, wish: &LocaleName, state: &CurrentSystemLocale) -> bool {
        state
            .0
            .as_ref()
            .is_some_and(|current| same_locale(current, wish))
    }

    fn plan(changes: &[(&Single, &LocaleName)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |_, locale| {
            RootAction::SetSystemLocale {
                locale: locale.clone(),
            }
        })
    }

    single_key!();

    fn summary(_item: &Single, wish: &LocaleName) -> Summary {
        Summary::of(SummaryCode::SystemLocale, wish.to_string())
    }
}
