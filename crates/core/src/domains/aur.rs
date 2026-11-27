//! Packages built from the AUR. Removing one is a removal of the packages
//! domain, as for any other installed package.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use toolbox_plan::{BuildRun, PackageName, PlannedAction, UserAction};
use toolbox_state::aur::order::AurOrder;

use super::{InstalledPackages, batch, staged};
use crate::domain::{Domain, Stage, Staged, Summary};
use crate::features::packages::aur::build;
use crate::{ChangeSet, SummaryCode};

pub struct AurPackages;

/// What the user wishes for an AUR package. `Paru` installs through paru's
/// own build. `Waiting` and `Built` are the split path, where the user builds
/// and the helper installs.
///
/// Only `paru` and `waiting` are ever written. The order of a `Built` wish is
/// read by the app itself, so a `built` wish from a file or the session bus
/// comes back as `Waiting`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AurWish {
    Paru,
    /// Its build order is not read yet, so it plans nothing.
    Waiting,
    Built(BuildOrder),
}

/// The order of a package, read for one apply.
///
/// `installed` holds the names of the order that were installed when it was
/// read, so an upgrade keeps their install reason. It has no serde form, so it
/// is never saved or sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildOrder {
    pub order: AurOrder,
    pub run: BuildRun,
    pub installed: BTreeSet<PackageName>,
}

impl BuildOrder {
    /// What the build does, for the review row of the package.
    pub fn outline(&self) -> build::Outline {
        build::outline(&self.order)
    }
}

/// The forms a wish is written in.
#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum Written {
    Paru,
    Waiting,
}

/// The forms a wish is read from. The body of `built` is dropped, so no
/// order from outside the app ever becomes a wish.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Read {
    Paru,
    Waiting,
    Built(IgnoredAny),
}

impl Serialize for AurWish {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Paru => Written::Paru.serialize(serializer),
            Self::Waiting | Self::Built(_) => Written::Waiting.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for AurWish {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Read::deserialize(deserializer)? {
            Read::Paru => Self::Paru,
            Read::Waiting | Read::Built(_) => Self::Waiting,
        })
    }
}

impl Domain for AurPackages {
    type Item = PackageName;
    type Wish = AurWish;
    type State = InstalledPackages;

    const ID: &'static str = "aur-packages";
    /// A build may need what the repository stage installs, and the DKMS
    /// stage may build a module an AUR package ships.
    const STAGE: Stage = Stage::new(300);
    const CODES: &'static [SummaryCode] = &[SummaryCode::AurInstall];

    fn fits(item: &PackageName, wish: &AurWish) -> bool {
        match wish {
            AurWish::Built(built) => build::mentions(&built.order, item),
            AurWish::Paru | AurWish::Waiting => true,
        }
    }

    fn is_satisfied(item: &PackageName, _wish: &AurWish, state: &InstalledPackages) -> bool {
        state.0.contains(item)
    }

    fn plan(changes: &[(&PackageName, &AurWish)]) -> Vec<Staged> {
        let names = changes
            .iter()
            .filter(|(_, wish)| matches!(wish, AurWish::Paru))
            .map(|(item, _)| (*item).clone())
            .collect();

        let mut split = split(changes);
        let mut planned = Vec::new();

        // Paru's install asks for the password through pkexec, so the review
        // of the split part must come before it.
        if split.first().is_some_and(is_review) {
            planned.push(split.remove(0));
        }

        planned.extend(staged([batch(Self::STAGE, names, |packages| {
            PlannedAction::User(UserAction::InstallAur { packages })
        })]));
        planned.extend(split);
        planned
    }

    named_key!();

    fn summary(item: &PackageName, _wish: &AurWish) -> Summary {
        Summary::of(SummaryCode::AurInstall, item.as_str())
    }
}

const fn is_review(step: &Staged) -> bool {
    matches!(
        step.action,
        PlannedAction::User(UserAction::ReviewAur { bases: _ })
    )
}

/// The queued packages whose build order is not read yet. A plan holds
/// back until they are read or taken out of the queue.
pub fn waiting_orders(changes: &ChangeSet) -> Vec<PackageName> {
    changes
        .queued::<AurPackages>()
        .into_iter()
        .filter(|(_, wish)| matches!(wish, AurWish::Waiting))
        .map(|(item, _)| item)
        .collect()
}

/// The split path. One `Waiting` wish holds all of it back, so a queue never
/// builds half. Built wishes of two runs plan nothing, as the run folder is
/// one per apply.
fn split(changes: &[(&PackageName, &AurWish)]) -> Vec<Staged> {
    let mut names = Vec::new();
    let mut orders = Vec::new();
    let mut installed = BTreeSet::new();
    let mut run: Option<&BuildRun> = None;

    for (item, wish) in changes {
        match wish {
            AurWish::Paru => {}
            AurWish::Waiting => return Vec::new(),
            AurWish::Built(built) => {
                if run.is_some_and(|known| *known != built.run) {
                    return Vec::new();
                }

                run = Some(&built.run);
                names.push((*item).clone());
                orders.push(built.order.clone());
                installed.extend(built.installed.iter().cloned());
            }
        }
    }

    let Some(run) = run else {
        return Vec::new();
    };

    match build::plan(&names, &orders, run, &installed) {
        Ok(planned) => planned
            .actions
            .into_iter()
            .map(|action| Staged::at(AurPackages::STAGE, action))
            .collect(),
        Err(_missing) => Vec::new(),
    }
}
