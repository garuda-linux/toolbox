//! What the action tests compare: each step as the Review page shows it.

use super::{RootAction, TargetList, UserAction};
use crate::UnitName;

pub fn shown(action: &RootAction) -> Vec<String> {
    action.procedure().iter().map(ToString::to_string).collect()
}

pub fn shown_user(action: &UserAction) -> Vec<String> {
    action.procedure().iter().map(ToString::to_string).collect()
}

pub fn units(names: &[&str]) -> TargetList<UnitName> {
    TargetList::new(names.iter().map(|name| UnitName::new(*name).unwrap())).unwrap()
}
