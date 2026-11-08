//! Why an upload failed, as codes the UI maps to translated text.

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum UploadFailure {
    /// The server was not reached: no network, DNS, TLS, a timeout.
    Network,
    /// The server said no, for example to a paste that is too large.
    Refused,
    /// The server answered something that is no paste.
    BadAnswer,
    Seal,
    /// The upload's thread stopped before it answered.
    Stopped,
}

impl UploadFailure {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}
