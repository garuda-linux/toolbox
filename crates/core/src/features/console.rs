//! The apply Console: why a plan stopped outside any step, and how one
//! that ended well reads. The UI maps the codes to translated text.

pub mod ending;

#[cfg(test)]
mod tests;

/// The helper or the steps that run as the user failed outside a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum HelperFailure {
    /// A step that needs a password came without one. Checked before the
    /// helper starts, which would wait for it forever.
    MissingSecret,
    Start,
    /// pkexec itself cannot run here, for example a copy that is not
    /// setuid root. The detail is its own words.
    Pkexec,
    /// The helper started without the pipes it is talked to through.
    NoPipes,
    Lost,
    /// The helper ended before it reported the plan done: killed or
    /// crashed.
    Ended,
    UserSteps,
}

impl HelperFailure {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}
