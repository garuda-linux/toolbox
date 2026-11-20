//! pacman's copies beside configuration files. One snapshot goes first,
//! so the merges roll back together.

#[cfg(test)]
mod tests;

use toolbox_plan::{
    CopyKind, MergeChoice, MergeCopy, MergeResolve, PlannedAction, Resolution, RootAction,
};
pub use toolbox_state::parts::MergeCopies;

use super::each;
use crate::SummaryCode;
use crate::domain::{Domain, Stage, Staged, Summary};

pub struct ConfigMerges;

impl Domain for ConfigMerges {
    type Item = MergeCopy;
    type State = MergeCopies;
    type Wish = MergeChoice;

    const ID: &'static str = "config-merges";
    const STAGE: Stage = Stage::new(Stage::FILES.rank() + 3);
    const CODES: &'static [SummaryCode] = &[
        SummaryCode::ConfigMergeDiscard,
        SummaryCode::ConfigMergeKeep,
        SummaryCode::ConfigMergeTake,
        SummaryCode::ConfigMergeWrite,
    ];

    fn fits(item: &MergeCopy, wish: &MergeChoice) -> bool {
        wish.fits(item)
    }

    /// A copy that is gone was resolved, by this queue or by hand.
    fn is_satisfied(item: &MergeCopy, _wish: &MergeChoice, state: &MergeCopies) -> bool {
        !state.0.contains(item)
    }

    fn plan(changes: &[(&MergeCopy, &MergeChoice)]) -> Vec<Staged> {
        let snapshot = (!changes.is_empty()).then(|| {
            Staged::at(
                Self::STAGE,
                PlannedAction::Root(RootAction::SnapshotBeforeMerges),
            )
        });

        snapshot
            .into_iter()
            .chain(each(Self::STAGE, changes, |copy, choice| {
                RootAction::ResolveMerge {
                    merge: MergeResolve {
                        copy: copy.clone(),
                        choice: choice.clone(),
                    },
                }
            }))
            .collect()
    }

    named_key!();

    fn summary(item: &MergeCopy, wish: &MergeChoice) -> Summary {
        let code = match wish.resolution {
            Resolution::KeepCurrent if item.kind() == CopyKind::New => SummaryCode::ConfigMergeKeep,
            Resolution::KeepCurrent => SummaryCode::ConfigMergeDiscard,
            Resolution::TakeCopy => SummaryCode::ConfigMergeTake,
            Resolution::Merged { text: _ } => SummaryCode::ConfigMergeWrite,
        };

        Summary::pair(code, item.as_str(), item.original().as_str())
    }
}
