#[cfg(test)]
mod tests;

pub use toolbox_state::accounts::{AccountLockFlags, Accounts, CurrentUser, Group, User};

use crate::Stage;

/// Where the account domains run around [`Stage::ACCOUNTS`], the login
/// shells, in this order. Each domain says why it sits where it does.
pub const NEW_GROUPS: Stage = Stage::new(Stage::ACCOUNTS.rank() - 20);
pub const MEMBERSHIPS: Stage = Stage::new(Stage::ACCOUNTS.rank() - 10);
pub const FULL_NAMES: Stage = Stage::new(Stage::ACCOUNTS.rank() + 10);
pub const HOME_FOLDERS: Stage = Stage::new(Stage::ACCOUNTS.rank() + 15);
pub const LOCKS: Stage = Stage::new(Stage::ACCOUNTS.rank() + 20);
pub const REMOVED_USERS: Stage = Stage::new(Stage::ACCOUNTS.rank() + 50);
pub const REMOVED_GROUPS: Stage = Stage::new(Stage::ACCOUNTS.rank() + 60);
