#[cfg(test)]
mod tests;

pub use toolbox_state::accounts::{AccountLockFlags, Accounts, CurrentUser, Group, User};

use crate::Stage;

/// Where the account domains run around [`Stage::ACCOUNTS`], the login
/// shells, in this order.
pub(super) const NEW_GROUPS: Stage = Stage::new(Stage::ACCOUNTS.rank() - 20);
pub(super) const MEMBERSHIPS: Stage = Stage::new(Stage::ACCOUNTS.rank() - 10);
pub(super) const FULL_NAMES: Stage = Stage::new(Stage::ACCOUNTS.rank() + 10);
pub(super) const HOME_FOLDERS: Stage = Stage::new(Stage::ACCOUNTS.rank() + 15);
pub(super) const LOCKS: Stage = Stage::new(Stage::ACCOUNTS.rank() + 20);
pub(super) const REMOVED_USERS: Stage = Stage::new(Stage::ACCOUNTS.rank() + 50);
pub(super) const REMOVED_GROUPS: Stage = Stage::new(Stage::ACCOUNTS.rank() + 60);
