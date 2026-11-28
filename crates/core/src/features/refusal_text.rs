//! What every page's refusal shares: a code from its strum name, and the
//! id of its text. Each id is written out in full, so grep and the catalog
//! checks see every one.

#[cfg(test)]
mod tests;

use toolbox_plan::Protection;

use super::ReasonCode;

/// A refusal and what its text names as `$name`.
///
/// A rule that refuses on account of another package, such as a protected
/// dependency, names that one, so the subject comes from the rule and not
/// from the click.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused<R, S = String> {
    pub refusal: R,
    pub subject: S,
}

pub trait RefusalText: Copy + Into<&'static str> {
    /// The id of the text that names this refusal, `None` for a refusal
    /// that no window words, because its page never shows it.
    fn text_id(self) -> Option<&'static str>;

    fn code(self) -> &'static str {
        self.into()
    }

    /// Why a protected package stays, the `$why` of its text.
    fn protection(self) -> Option<Protection> {
        None
    }
}

impl<R, S> Refused<R, S> {
    pub const fn new(refusal: R, subject: S) -> Self {
        Self { refusal, subject }
    }
}

impl RefusalText for super::boot::Refusal {
    fn text_id(self) -> Option<&'static str> {
        Some(match self {
            Self::Unavailable => "boot-refused-unavailable",
            Self::CustomCommandLine => "boot-refused-custom-command-line",
            Self::TooManyParameters => "boot-refused-too-many-parameters",
            Self::UnknownTheme => "boot-refused-unknown-theme",
            Self::NoKernelCmdline => "boot-refused-no-kernel-cmdline",
            Self::CustomKernelCmdline => "boot-refused-custom-kernel-cmdline",
            Self::ProtectedParameter => "boot-refused-protected-parameter",
        })
    }
}

impl RefusalText for super::bootrepair::Refusal {
    fn text_id(self) -> Option<&'static str> {
        Some(match self {
            Self::NotOffered => "repair-refused-not-offered",
            Self::Unavailable => "repair-refused-unavailable",
            Self::UnknownSystem => "repair-refused-unknown-system",
            Self::InUse => "repair-refused-in-use",
            Self::UnknownTarget => "repair-refused-unknown-target",
            Self::UnknownLayout => "repair-refused-unknown-layout",
            Self::NoPassphrase => "repair-refused-empty",
        })
    }
}

impl RefusalText for super::kernels::Refusal {
    fn text_id(self) -> Option<&'static str> {
        match self {
            Self::Package(refusal) => refusal.text_id(),
            Self::Running => Some("kernels-refusal-running"),
            Self::RunningUnknown => Some("kernels-refusal-running-unknown"),
            Self::LastKernel => Some("kernels-refusal-last-kernel"),
        }
    }

    fn protection(self) -> Option<Protection> {
        match self {
            Self::Package(refusal) => refusal.protection(),
            Self::Running | Self::RunningUnknown | Self::LastKernel => None,
        }
    }
}

impl RefusalText for super::locales::Refusal {
    fn text_id(self) -> Option<&'static str> {
        match self {
            Self::Package(refusal) => refusal.text_id(),
            Self::SystemLocale => Some("locales-refused-system-locale"),
            Self::LastLocale => Some("locales-refused-last-locale"),
            Self::NotEnabled => Some("locales-refused-not-enabled"),
        }
    }

    fn protection(self) -> Option<Protection> {
        match self {
            Self::Package(refusal) => refusal.protection(),
            Self::SystemLocale | Self::LastLocale | Self::NotEnabled => None,
        }
    }
}

impl RefusalText for super::network::Refusal {
    fn text_id(self) -> Option<&'static str> {
        match self {
            Self::Package(refusal) => refusal.text_id(),
            Self::OtherFile => Some("network-refused-other-file"),
            Self::Unavailable => Some("network-refused-unavailable"),
            Self::Busy => Some("network-refused-busy"),
        }
    }

    fn protection(self) -> Option<Protection> {
        match self {
            Self::Package(refusal) => refusal.protection(),
            Self::OtherFile | Self::Unavailable | Self::Busy => None,
        }
    }
}

impl RefusalText for super::users::Refusal {
    fn text_id(self) -> Option<&'static str> {
        Some(match self {
            Self::Unavailable => "users-refused-unavailable",
            Self::UnknownUser => "users-refused-unknown-user",
            Self::UnknownGroup => "users-refused-unknown-group",
            Self::Root => "users-refused-root",
            Self::CurrentUser => "users-refused-current-user",
            Self::SystemAccount => "users-refused-system-account",
            Self::OutsideUserRange => "users-refused-outside-user-range",
            Self::LastAdmin => "users-refused-last-admin",
            Self::LastAdminLock => "users-refused-last-admin-lock",
            Self::LockStateUnknown => "users-refused-lock-state-unknown",
            Self::PrimaryGroup => "users-refused-primary-group",
            Self::SystemGroup => "users-refused-system-group",
            Self::UserExists => "users-refused-user-exists",
            Self::GroupExists => "users-refused-group-exists",
            Self::ShellNotListed => "users-refused-shell-not-listed",
            Self::UidInUse => "users-refused-uid-in-use",
            Self::UidOutOfRange => "users-refused-uid-out-of-range",
            Self::HomeTaken => "users-refused-home-taken",
            Self::Busy => "users-refused-busy",
        })
    }
}

/// A protected package shares one text with the toggle pages.
impl RefusalText for super::packages::queue::Refusal {
    fn text_id(self) -> Option<&'static str> {
        Some(match self {
            Self::Protected(_) => "protected-refused",
            Self::TakesAlong(_) => "protected-along",
            Self::NotOffered => "packages-refused-not-offered",
            Self::Unread => "packages-refused-unread",
            Self::RunningKernel => "packages-refused-running-kernel",
            Self::RunningUnknown => "packages-refused-running-unknown",
            Self::LastKernel => "packages-refused-last-kernel",
            Self::Conflicts => "packages-refused-conflicts",
        })
    }

    fn protection(self) -> Option<Protection> {
        match self {
            Self::Protected(why) | Self::TakesAlong(why) => Some(why),
            Self::NotOffered
            | Self::Unread
            | Self::RunningKernel
            | Self::RunningUnknown
            | Self::LastKernel
            | Self::Conflicts => None,
        }
    }
}

/// A switch that meets another page's rule shows that page's text. The
/// Unknown and Unmet refusals have none: the Toggles and Discover pages
/// never word them, a switch that is unknown or unmet rests.
impl RefusalText for super::toggles::Refusal {
    fn text_id(self) -> Option<&'static str> {
        match self {
            Self::Account(refusal) => Some(account_id(refusal)),
            Self::Kernel(refusal) => refusal.text_id(),
            Self::Protected(_) => Some("protected-refused"),
            Self::TakesAlong(_) => Some("protected-along"),
            Self::Unknown | Self::Unmet => None,
        }
    }

    fn protection(self) -> Option<Protection> {
        match self {
            Self::Kernel(refusal) => refusal.protection(),
            Self::Protected(why) | Self::TakesAlong(why) => Some(why),
            Self::Account(_) | Self::Unknown | Self::Unmet => None,
        }
    }
}

/// The boot edit that asks first has no text: the Boot page asks in a
/// question and never words that refusal.
impl RefusalText for super::boot::edit::EditCode {
    fn text_id(self) -> Option<&'static str> {
        match self {
            Self::Refused(refusal) => refusal.text_id(),
            Self::Rule(reason) => Some(boot_rule_id(reason)),
            Self::NeedsConfirm => None,
        }
    }
}

impl RefusalText for super::configs::Refusal {
    fn text_id(self) -> Option<&'static str> {
        Some(match self {
            Self::Unknown => "configs-refused-unknown",
            Self::Busy => "configs-refused-busy",
            Self::ReadOnly => "configs-refused-read-only",
        })
    }
}

impl RefusalText for super::configs::session::ProblemCode {
    fn text_id(self) -> Option<&'static str> {
        Some(match self {
            Self::Dismissed => "configs-problem-dismissed",
            Self::Denied => "configs-problem-denied",
            Self::NoAgent => "configs-problem-no-agent",
            Self::HelperMissing => "configs-problem-helper-missing",
            Self::Broken => "configs-problem-broken",
            Self::Missing => "configs-problem-missing",
            Self::Link => "configs-problem-link",
            Self::Dangling => "configs-problem-dangling",
            Self::Outside => "configs-problem-outside",
            Self::NotRegular => "configs-problem-not-regular",
            Self::TooLarge => "configs-problem-too-large",
            Self::NotText => "configs-problem-not-text",
            Self::NoEditor => "configs-problem-no-editor",
            Self::Busy => "configs-problem-busy",
            Self::EditorFailed => "configs-problem-editor-failed",
            Self::WriteFailed => "configs-problem-write-failed",
            Self::CopyFailed => "configs-problem-copy-failed",
            Self::ReadFailed => "configs-problem-read-failed",
            Self::NoHome => "configs-problem-no-home",
            Self::Cancelled => "configs-problem-cancelled",
            Self::Failed => "configs-problem-failed",
            Self::HelperStart => "configs-problem-helper-start",
            Self::HelperTimedOut => "configs-problem-helper-timed-out",
            Self::HelperTooLong => "configs-problem-helper-too-long",
            Self::HelperPkexec => "configs-problem-helper-pkexec",
            Self::HelperEnded => "configs-problem-helper-ended",
            Self::HelperUnreadable => "configs-problem-helper-unreadable",
            Self::HelperStopped => "configs-problem-helper-stopped",
        })
    }
}

/// A field's rule shows the text of that field, a refusal the page's own.
impl RefusalText for super::users::input::InputProblem {
    fn text_id(self) -> Option<&'static str> {
        match self {
            Self::Name(reason) => Some(user_name_id(reason)),
            Self::FullName(rule) => Some(full_name_id(rule)),
            Self::Home(rule) => Some(home_id(rule)),
            Self::Refused(refusal) => refusal.text_id(),
            Self::UidInvalid => Some("users-refused-uid-invalid"),
        }
    }
}

impl From<super::users::input::InputProblem> for &'static str {
    fn from(problem: super::users::input::InputProblem) -> Self {
        use super::users::input::InputProblem;

        match problem {
            InputProblem::Name(rule) => rule.code(),
            InputProblem::FullName(rule) => rule.as_str(),
            InputProblem::Home(rule) => rule.as_str(),
            InputProblem::Refused(refusal) => refusal.code(),
            InputProblem::UidInvalid => "uid-invalid",
        }
    }
}

/// The text of why `protection` keeps a package, the `$why` of
/// `protected-refused` and `protected-along`.
pub const fn protection_text_id(protection: Protection) -> &'static str {
    match protection {
        Protection::Boot => "protected-boot",
        Protection::Root => "protected-root",
        Protection::Login => "protected-login",
        Protection::Network => "protected-network",
        Protection::PackageManager => "protected-package-manager",
        Protection::Desktop => "protected-desktop",
    }
}

/// The account rule a membership change meets, in the words of the
/// Users page's refusal.
const fn account_id(refusal: super::users::Refusal) -> &'static str {
    use super::users::Refusal;

    match refusal {
        Refusal::Unavailable => "account-refusal-unavailable",
        Refusal::UnknownUser => "account-refusal-unknown-user",
        Refusal::UnknownGroup => "account-refusal-unknown-group",
        Refusal::Root => "account-refusal-root",
        Refusal::CurrentUser => "account-refusal-current-user",
        Refusal::SystemAccount => "account-refusal-system-account",
        Refusal::OutsideUserRange => "account-refusal-outside-user-range",
        Refusal::LastAdmin => "account-refusal-last-admin",
        Refusal::LastAdminLock => "account-refusal-last-admin-lock",
        Refusal::LockStateUnknown => "account-refusal-lock-state-unknown",
        Refusal::PrimaryGroup => "account-refusal-primary-group",
        Refusal::SystemGroup => "account-refusal-system-group",
        Refusal::UserExists => "account-refusal-user-exists",
        Refusal::GroupExists => "account-refusal-group-exists",
        Refusal::ShellNotListed => "account-refusal-shell-not-listed",
        Refusal::UidInUse => "account-refusal-uid-in-use",
        Refusal::UidOutOfRange => "account-refusal-uid-out-of-range",
        Refusal::HomeTaken => "account-refusal-home-taken",
        Refusal::Busy => "account-refusal-busy",
    }
}

/// The boot values that break a rule name it in the boot page's words.
const fn boot_rule_id(reason: ReasonCode) -> &'static str {
    match reason {
        ReasonCode::Empty => "boot-refused-empty",
        ReasonCode::TooLong => "boot-refused-too-long",
        ReasonCode::InvalidStart | ReasonCode::InvalidCharacter => "boot-refused-invalid-character",
        ReasonCode::InvalidPath => "boot-refused-invalid-path",
        ReasonCode::WrongType => "boot-refused-wrong-type",
        ReasonCode::Invalid => "boot-refused-invalid",
    }
}

const fn user_name_id(reason: ReasonCode) -> &'static str {
    match reason {
        ReasonCode::Empty => "users-name-empty",
        ReasonCode::TooLong => "users-name-too-long",
        ReasonCode::InvalidStart => "users-name-invalid-start",
        ReasonCode::InvalidCharacter => "users-name-invalid-character",
        ReasonCode::InvalidPath | ReasonCode::WrongType | ReasonCode::Invalid => {
            "users-name-invalid"
        }
    }
}

const fn full_name_id(rule: toolbox_plan::InvalidFullName) -> &'static str {
    use toolbox_plan::InvalidFullName;

    match rule {
        InvalidFullName::TooLong { max: _ } => "users-full-name-too-long",
        InvalidFullName::InvalidCharacter(_) => "users-full-name-invalid-character",
        InvalidFullName::LeadingDash => "users-full-name-leading-dash",
        InvalidFullName::OuterBlank => "users-full-name-outer-blank",
    }
}

const fn home_id(rule: toolbox_plan::InvalidHomeDirectory) -> &'static str {
    use toolbox_plan::InvalidHomeDirectory;

    match rule {
        InvalidHomeDirectory::Empty => "users-home-empty",
        InvalidHomeDirectory::TooLong { max: _ } => "users-home-too-long",
        InvalidHomeDirectory::NotAbsolute => "users-home-not-absolute",
        InvalidHomeDirectory::InvalidCharacter(_) => "users-home-invalid-character",
        InvalidHomeDirectory::InvalidComponent => "users-home-invalid-component",
        InvalidHomeDirectory::OuterBlank => "users-home-outer-blank",
        InvalidHomeDirectory::OutsideRoots => "users-home-outside-roots",
    }
}
