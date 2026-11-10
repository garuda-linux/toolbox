//! The catalog toggles queue it for the app's user, the Users page for any account.

use serde::{Deserialize, Serialize};
use toolbox_plan::{GroupName, RootAction, UserName};

use super::accounts::{self, Accounts};
use super::each;
use crate::SummaryCode;
use crate::domain::{Domain, Stage, Staged, Summary, Toggle};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Membership {
    pub user: UserName,
    pub group: GroupName,
}

impl Membership {
    pub const fn new(user: UserName, group: GroupName) -> Self {
        Self { user, group }
    }
}

pub struct Groups;

impl Domain for Groups {
    type Item = Membership;
    type State = Accounts;
    type Wish = Toggle;

    const ID: &'static str = "groups";
    const STAGE: Stage = accounts::MEMBERSHIPS;

    fn is_satisfied(item: &Membership, wish: &Toggle, state: &Accounts) -> bool {
        state.is_member(&item.user, &item.group) == wish.is_on()
    }

    /// gpasswd takes one user and one group per call.
    fn plan(changes: &[(&Membership, &Toggle)]) -> Vec<Staged> {
        each(Self::STAGE, changes, |membership, wish| {
            let user = membership.user.clone();
            let group = membership.group.clone();

            if wish.is_on() {
                RootAction::AddToGroup { user, group }
            } else {
                RootAction::RemoveFromGroup { user, group }
            }
        })
    }

    fn key(item: &Membership) -> String {
        format!("{} {}", item.user, item.group)
    }

    fn summary(item: &Membership, wish: &Toggle) -> Summary {
        let code = if wish.is_on() {
            SummaryCode::GroupJoin
        } else {
            SummaryCode::GroupLeave
        };

        Summary::pair(code, item.user.as_str(), item.group.as_str())
    }
}
