use toolbox_plan::RootAction;

use super::super::program_name;

#[test]
fn a_step_is_named_by_its_first_program() {
    assert_eq!(
        program_name(&RootAction::RefreshKeyring.procedure()),
        "pacman-key"
    );
    assert_eq!(
        program_name(&RootAction::CleanPackageCache.procedure()),
        "paccache"
    );
}

#[test]
fn a_step_the_helper_does_itself_is_a_builtin() {
    assert_eq!(
        program_name(&RootAction::RemoveDatabaseLock.procedure()),
        "builtin"
    );
}
