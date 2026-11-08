//! The English line of each page rule. The command line has no window to
//! show the page's text, so these say the same as `app/gui/i18n/en-US`
//! in one line. Fluent stays out of the CLI on purpose.

#[cfg(test)]
mod tests;

use toolbox_core::features::boot::Refusal as BootRefusal;
use toolbox_core::features::boot::edit::EditProblem;
use toolbox_core::features::bootrepair::Refusal as RepairRefusal;
use toolbox_core::features::locales::Refusal as LocaleRefusal;
use toolbox_core::features::network::Refusal as DriverRefusal;
use toolbox_core::features::users::Refusal as AccountRefusal;
use toolbox_plan::Reason;

/// A switch the catalog lacks. The commands check the id first, so this
/// stays a fallback.
pub const SWITCH_UNKNOWN: &str = "the catalog has no such switch";

pub const SWITCH_UNMET: &str = "this switch needs another one that is off, turn that on first";

pub const fn repair(refusal: RepairRefusal) -> &'static str {
    match refusal {
        RepairRefusal::NotOffered => "boot repair is only offered on the live system",
        RepairRefusal::Unavailable => "the disks could not be read, so nothing can be repaired",
        RepairRefusal::UnknownSystem => {
            "this system is not in the list anymore, see `bootrepair show`"
        }
        RepairRefusal::InUse => "the running system uses this disk, so nothing is written to it",
        RepairRefusal::UnknownTarget => "choose where the boot loader goes first",
        RepairRefusal::UnknownLayout => "choose where the system lies first",
        RepairRefusal::NoPassphrase => "enter the passphrase of the encrypted partition",
    }
}

pub const fn account(refusal: AccountRefusal) -> &'static str {
    match refusal {
        AccountRefusal::Unavailable => "the user accounts of this system could not be read",
        AccountRefusal::UnknownUser => "there is no user by this name",
        AccountRefusal::UnknownGroup => {
            "one of the groups does not exist, or is about to be removed"
        }
        AccountRefusal::Root => "this is a root account, Garuda Toolbox does not change it",
        AccountRefusal::CurrentUser => {
            "this is the account you are logged in with, it cannot lock or remove itself"
        }
        AccountRefusal::SystemAccount => {
            "this is a system account, only accounts of people can be removed"
        }
        AccountRefusal::OutsideUserRange => {
            "this account has a user ID outside 1000 to 60000, so it stays as it is"
        }
        AccountRefusal::LastAdmin => {
            "this is the last member of the groups wheel and sudo, \
             without one nobody could administer this system"
        }
        AccountRefusal::LastAdminLock => {
            "this is the last member of the groups wheel and sudo, \
             with a locked password nobody could administer this system"
        }
        AccountRefusal::LockStateUnknown => {
            "this account cannot be locked while it is unknown which admins are locked"
        }
        AccountRefusal::PrimaryGroup => "this is the primary group of a user, so it stays",
        AccountRefusal::SystemGroup => "this is a system group, installed programs rely on it",
        AccountRefusal::UserExists => "a user by this name exists already",
        AccountRefusal::GroupExists => "a group by this name exists already",
        AccountRefusal::ShellNotListed => "this shell is not listed in /etc/shells",
        AccountRefusal::UidInUse => "another account already has this user ID",
        AccountRefusal::UidOutOfRange => {
            "accounts of people take a user ID from 1000 to 60000, \
             system accounts one below 1000"
        }
        AccountRefusal::HomeTaken => "another account already has this home folder",
    }
}

pub const fn locale(refusal: LocaleRefusal) -> &'static str {
    match refusal {
        LocaleRefusal::SystemLocale => {
            "this is the system language, choose another system language before you disable it"
        }
        LocaleRefusal::LastLocale => "at least one locale must stay enabled",
        LocaleRefusal::NotEnabled => "this locale is not enabled, enable it first",
    }
}

pub const fn driver(refusal: DriverRefusal) -> &'static str {
    match refusal {
        DriverRefusal::OtherFile => {
            "another driver settings file in modprobe.d blacklists this driver, \
             Garuda Toolbox only changes its own file"
        }
        DriverRefusal::Unavailable => "the drivers of this system could not be read",
    }
}

/// `value` is the value as the user gave it, which the page's text names.
pub fn boot_edit(problem: EditProblem, value: &str) -> String {
    match problem {
        EditProblem::Refused(BootRefusal::Unavailable) => {
            "this system has no /etc/default/grub, so there is nothing to change".to_owned()
        }
        EditProblem::Refused(BootRefusal::CustomCommandLine) => {
            "the kernel command line is not changed here, edit /etc/default/grub instead".to_owned()
        }
        EditProblem::Refused(BootRefusal::TooManyParameters) => {
            "the kernel command line is full".to_owned()
        }
        EditProblem::Refused(BootRefusal::UnknownTheme) => {
            format!("`{value}` is not an installed Plymouth theme")
        }
        EditProblem::Refused(BootRefusal::NoKernelCmdline) => {
            "this system has no /etc/kernel/cmdline, its boot entries carry their own parameters"
                .to_owned()
        }
        EditProblem::Refused(BootRefusal::CustomKernelCmdline) => {
            "the kernel command line is not changed here, edit /etc/kernel/cmdline instead"
                .to_owned()
        }
        EditProblem::Refused(BootRefusal::ProtectedParameter) => {
            format!("`{value}` stays, without it the system finds no root file system to start")
        }
        EditProblem::Invalid(invalid) => match invalid.reason() {
            Reason::Empty => "nothing was entered".to_owned(),
            Reason::TooLong { max: _ } => format!("`{value}` is too long"),
            Reason::InvalidStart(_) | Reason::InvalidCharacter(_) => format!(
                "`{value}` holds a character that is not allowed here, \
                 such as a blank, a quote, a backslash or a dollar sign"
            ),
            Reason::InvalidComponent => format!("`{value}` is not a full path to a file"),
            Reason::WrongFileType => format!("`{value}` is not a file of the right type"),
            Reason::UnknownUnitType | Reason::UnknownDevice => {
                format!("`{value}` cannot be used here")
            }
        },
        EditProblem::OutOfRange => format!("`{value}` cannot be used here"),
    }
}

/// `name` is the account or the group the Users page's text names.
pub fn membership(refusal: AccountRefusal, name: &str) -> String {
    format!("`{name}`: {}", account(refusal))
}
