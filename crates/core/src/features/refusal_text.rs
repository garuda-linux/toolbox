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
}

impl RefusalText for super::locales::Refusal {
    const TEXT_PREFIX: &'static str = "locales-refused";
}

impl RefusalText for super::network::Refusal {
    const TEXT_PREFIX: &'static str = "network-refused";
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
}

impl RefusalText for super::boot::edit::EditCode {
    const TEXT_PREFIX: &'static str = "boot-refused";
}

impl RefusalText for super::configs::Refusal {
    const TEXT_PREFIX: &'static str = "configs-refused";
}

/// The text of why `protection` keeps a package, the `$why` of
/// `protected-refused` and `protected-along`.
pub fn protection_text_id(protection: Protection) -> String {
    format!("protected-{}", protection.as_str())
}
