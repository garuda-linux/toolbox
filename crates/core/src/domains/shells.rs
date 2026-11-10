#[cfg(test)]
mod tests;

use toolbox_plan::{RootAction, ShellPath, UserName};
pub use toolbox_state::parts::{AvailableShells, CurrentShells};

use super::each;
use crate::SummaryCode;
use crate::domain::{Domain, Stage, Staged, Summary};

pub struct LoginShells;

impl Domain for LoginShells {
    type Item = UserName;
    type State = CurrentShells;
    type Wish = ShellPath;

    const ID: &'static str = "login-shells";
    const STAGE: Stage = Stage::ACCOUNTS;
    const CODES: &'static [SummaryCode] = &[SummaryCode::LoginShell];

    fn is_satisfied(item: &UserName, wish: &ShellPath, state: &CurrentShells) -> bool {
        state.0.get(item) == Some(wish)
    }

    /// chsh takes one account per call.
    fn plan(changes: &[(&UserName, &ShellPath)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |user, shell| RootAction::SetShell {
            user: user.clone(),
            shell: shell.clone(),
        })
    }

    named_key!();

    /// The summary has no access to the snapshot, so the page shows the
    /// current shell beside it.
    fn summary(item: &UserName, wish: &ShellPath) -> Summary {
        Summary::pair(SummaryCode::LoginShell, item.as_str(), wish.as_str())
    }
}
