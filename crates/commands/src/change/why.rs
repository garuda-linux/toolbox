//! The one line of each page rule. The command line has no window to show
//! the page's text, so these say the same in one line, from the `cli-*`
//! texts in `crates/services/i18n`. The command line speaks English.

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use toolbox_core::features::boot::Refusal as BootRefusal;
use toolbox_core::features::boot::edit::EditProblem;
use toolbox_core::features::bootrepair::Refusal as RepairRefusal;
use toolbox_core::features::locales::Refusal as LocaleRefusal;
use toolbox_core::features::network::{Refusal as DriverRefusal, SwitchRefusal};
use toolbox_core::features::users::Refusal as AccountRefusal;
use toolbox_plan::Reason;
use toolbox_services::i18n::{FALLBACK, text_in, text_with_in};

/// A switch the catalog lacks. The commands check the id first, so this
/// stays a fallback.
pub fn switch_unknown() -> Cow<'static, str> {
    text_in(FALLBACK, "cli-switch-unknown")
}

pub fn switch_unmet() -> Cow<'static, str> {
    text_in(FALLBACK, "cli-switch-unmet")
}

pub fn repair(refusal: RepairRefusal) -> Cow<'static, str> {
    text_in(FALLBACK, repair_id(refusal))
}

pub fn account(refusal: AccountRefusal) -> Cow<'static, str> {
    text_in(FALLBACK, account_id(refusal))
}

pub fn no_admin_group() -> Cow<'static, str> {
    text_in(FALLBACK, "cli-account-no-admin-group")
}

pub fn locale(refusal: LocaleRefusal) -> Cow<'static, str> {
    let id = match refusal {
        LocaleRefusal::SystemLocale => "cli-locale-system-locale",
        LocaleRefusal::LastLocale => "cli-locale-last-locale",
        LocaleRefusal::NotEnabled => "cli-locale-not-enabled",
    };

    text_in(FALLBACK, id)
}

pub fn driver(refusal: DriverRefusal) -> Cow<'static, str> {
    let id = match refusal {
        DriverRefusal::OtherFile => "cli-driver-other-file",
        DriverRefusal::Unavailable => "cli-driver-unavailable",
    };

    text_in(FALLBACK, id)
}

pub fn driver_switch(refusal: SwitchRefusal) -> Cow<'static, str> {
    let id = match refusal {
        SwitchRefusal::NotListed => "cli-driver-not-listed",
        SwitchRefusal::AlreadyLoaded => "cli-driver-loaded",
        SwitchRefusal::NotLoaded => "cli-driver-not-loaded",
    };

    text_in(FALLBACK, id)
}

/// `value` is the value as the user gave it, which the page's text names.
pub fn boot_edit(problem: EditProblem, value: &str) -> String {
    text_with_in(&FALLBACK, boot_edit_id(problem), &[("value", value)]).into_owned()
}

/// `name` is the account or the group the Users page's text names.
pub fn membership(refusal: AccountRefusal, name: &str) -> String {
    let why = account(refusal);
    text_with_in(
        &FALLBACK,
        "cli-membership",
        &[("name", name), ("why", &why)],
    )
    .into_owned()
}

const fn repair_id(refusal: RepairRefusal) -> &'static str {
    match refusal {
        RepairRefusal::NotOffered => "cli-repair-not-offered",
        RepairRefusal::Unavailable => "cli-repair-unavailable",
        RepairRefusal::UnknownSystem => "cli-repair-unknown-system",
        RepairRefusal::InUse => "cli-repair-in-use",
        RepairRefusal::UnknownTarget => "cli-repair-unknown-target",
        RepairRefusal::UnknownLayout => "cli-repair-unknown-layout",
        RepairRefusal::NoPassphrase => "cli-repair-no-passphrase",
    }
}

const fn account_id(refusal: AccountRefusal) -> &'static str {
    match refusal {
        AccountRefusal::Unavailable => "cli-account-unavailable",
        AccountRefusal::UnknownUser => "cli-account-unknown-user",
        AccountRefusal::UnknownGroup => "cli-account-unknown-group",
        AccountRefusal::Root => "cli-account-root",
        AccountRefusal::CurrentUser => "cli-account-current-user",
        AccountRefusal::SystemAccount => "cli-account-system-account",
        AccountRefusal::OutsideUserRange => "cli-account-outside-user-range",
        AccountRefusal::LastAdmin => "cli-account-last-admin",
        AccountRefusal::LastAdminLock => "cli-account-last-admin-lock",
        AccountRefusal::LockStateUnknown => "cli-account-lock-state-unknown",
        AccountRefusal::PrimaryGroup => "cli-account-primary-group",
        AccountRefusal::SystemGroup => "cli-account-system-group",
        AccountRefusal::UserExists => "cli-account-user-exists",
        AccountRefusal::GroupExists => "cli-account-group-exists",
        AccountRefusal::ShellNotListed => "cli-account-shell-not-listed",
        AccountRefusal::UidInUse => "cli-account-uid-in-use",
        AccountRefusal::UidOutOfRange => "cli-account-uid-out-of-range",
        AccountRefusal::HomeTaken => "cli-account-home-taken",
    }
}

fn boot_edit_id(problem: EditProblem) -> &'static str {
    match problem {
        EditProblem::NeedsConfirm => "cli-boot-needs-confirm",
        EditProblem::Refused(BootRefusal::Unavailable) => "cli-boot-unavailable",
        EditProblem::Refused(BootRefusal::CustomCommandLine) => "cli-boot-custom-command-line",
        EditProblem::Refused(BootRefusal::TooManyParameters) => "cli-boot-too-many-parameters",
        EditProblem::Refused(BootRefusal::UnknownTheme) => "cli-boot-unknown-theme",
        EditProblem::Refused(BootRefusal::NoKernelCmdline) => "cli-boot-no-kernel-cmdline",
        EditProblem::Refused(BootRefusal::CustomKernelCmdline) => "cli-boot-custom-kernel-cmdline",
        EditProblem::Refused(BootRefusal::ProtectedParameter) => "cli-boot-protected-parameter",
        EditProblem::Invalid(invalid) => match invalid.reason() {
            Reason::Empty => "cli-boot-empty",
            Reason::TooLong { max: _ } => "cli-boot-too-long",
            Reason::InvalidStart(_) | Reason::InvalidCharacter(_) => "cli-boot-invalid-character",
            Reason::InvalidComponent => "cli-boot-invalid-component",
            Reason::WrongFileType => "cli-boot-wrong-file-type",
            Reason::UnknownUnitType | Reason::UnknownDevice => "cli-boot-not-usable",
        },
        EditProblem::OutOfRange => "cli-boot-not-usable",
    }
}
