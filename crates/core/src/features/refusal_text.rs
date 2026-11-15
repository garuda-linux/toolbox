//! What every page's refusal shares: a code from its strum name, and the
//! id of its text, which the page's prefix keeps apart from other pages'.

#[cfg(test)]
mod tests;

use toolbox_plan::Protection;

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
    /// Such as `kernels-refusal`, without the last hyphen.
    const TEXT_PREFIX: &'static str;

    fn code(self) -> &'static str {
        self.into()
    }

    fn text_id(self) -> String {
        format!("{}-{}", Self::TEXT_PREFIX, self.code())
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
    const TEXT_PREFIX: &'static str = "boot-refused";
}

impl RefusalText for super::bootrepair::Refusal {
    const TEXT_PREFIX: &'static str = "repair-refused";
}

impl RefusalText for super::kernels::Refusal {
    const TEXT_PREFIX: &'static str = "kernels-refusal";

    fn text_id(self) -> String {
        match self {
            Self::Package(refusal) => refusal.text_id(),
            Self::Running | Self::RunningUnknown | Self::LastKernel => {
                format!("{}-{}", Self::TEXT_PREFIX, self.code())
            }
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
    const TEXT_PREFIX: &'static str = "locales-refused";

    fn text_id(self) -> String {
        match self {
            Self::Package(refusal) => refusal.text_id(),
            Self::SystemLocale | Self::LastLocale | Self::NotEnabled => {
                format!("{}-{}", Self::TEXT_PREFIX, self.code())
            }
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
    const TEXT_PREFIX: &'static str = "network-refused";

    fn text_id(self) -> String {
        match self {
            Self::Package(refusal) => refusal.text_id(),
            Self::OtherFile | Self::Unavailable | Self::Busy => {
                format!("{}-{}", Self::TEXT_PREFIX, self.code())
            }
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
    const TEXT_PREFIX: &'static str = "users-refused";
}

/// A protected package shares one text with the toggle pages.
impl RefusalText for super::packages::queue::Refusal {
    const TEXT_PREFIX: &'static str = "packages-refused";

    fn text_id(self) -> String {
        match self {
            Self::Protected(_) => "protected-refused".to_owned(),
            Self::TakesAlong(_) => "protected-along".to_owned(),
            Self::NotOffered
            | Self::Unread
            | Self::RunningKernel
            | Self::RunningUnknown
            | Self::LastKernel => format!("{}-{}", Self::TEXT_PREFIX, self.code()),
        }
    }

    fn protection(self) -> Option<Protection> {
        match self {
            Self::Protected(why) | Self::TakesAlong(why) => Some(why),
            Self::NotOffered
            | Self::Unread
            | Self::RunningKernel
            | Self::RunningUnknown
            | Self::LastKernel => None,
        }
    }
}

/// A switch that meets another page's rule shows that page's text.
impl RefusalText for super::toggles::Refusal {
    const TEXT_PREFIX: &'static str = "toggles-refused";

    fn text_id(self) -> String {
        match self {
            Self::Account(refusal) => format!("account-refusal-{}", refusal.code()),
            Self::Kernel(refusal) => refusal.text_id(),
            Self::Protected(_) => "protected-refused".to_owned(),
            Self::TakesAlong(_) => "protected-along".to_owned(),
            Self::Unknown | Self::Unmet => format!("{}-{}", Self::TEXT_PREFIX, self.code()),
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

impl RefusalText for super::boot::edit::EditCode {
    const TEXT_PREFIX: &'static str = "boot-refused";
}

impl RefusalText for super::configs::Refusal {
    const TEXT_PREFIX: &'static str = "configs-refused";
}

impl RefusalText for super::configs::session::ProblemCode {
    const TEXT_PREFIX: &'static str = "configs-problem";
}

/// A field's rule shows the text of that field, a refusal the page's own.
impl RefusalText for super::users::input::InputProblem {
    const TEXT_PREFIX: &'static str = "users-refused";

    fn text_id(self) -> String {
        match self {
            Self::Name(rule) => format!("users-name-{}", rule.code()),
            Self::FullName(rule) => format!("users-full-name-{}", rule.as_str()),
            // Its code already starts with `home-`.
            Self::Home(rule) => format!("users-{}", rule.as_str()),
            Self::Refused(refusal) => refusal.text_id(),
            Self::UidInvalid => format!("{}-{}", Self::TEXT_PREFIX, self.code()),
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
pub fn protection_text_id(protection: Protection) -> String {
    format!("protected-{}", protection.as_str())
}
