//! The kernel command line as a list of words, so the page can add and
//! remove single parameters and no word can leave its quotes.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::KernelParameter;

/// More than any real command line holds.
const MAX_PARAMETERS: usize = 128;

/// The words joined by blanks, as they land in the file.
const MAX_BYTES: usize = 1024;

/// A list longer than a command line holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidKernelParameters {
    #[error("more than {max} kernel parameters")]
    TooMany { max: usize },
    #[error("a kernel command line longer than {max} bytes")]
    TooLong { max: usize },
}

/// `GRUB_CMDLINE_LINUX_DEFAULT` as its words, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<KernelParameter>", into = "Vec<KernelParameter>")]
pub struct KernelParameters(Vec<KernelParameter>);

impl KernelParameters {
    /// # Errors
    ///
    /// [`InvalidKernelParameters`] for more than `MAX_PARAMETERS` or
    /// `MAX_BYTES`.
    pub fn new(parameters: Vec<KernelParameter>) -> Result<Self, InvalidKernelParameters> {
        if parameters.len() > MAX_PARAMETERS {
            return Err(InvalidKernelParameters::TooMany {
                max: MAX_PARAMETERS,
            });
        }

        let blanks = parameters.len().saturating_sub(1);
        let bytes = parameters
            .iter()
            .map(|parameter| parameter.as_str().len())
            .sum::<usize>()
            + blanks;

        if bytes > MAX_BYTES {
            return Err(InvalidKernelParameters::TooLong { max: MAX_BYTES });
        }

        Ok(Self(parameters))
    }

    /// The words of a command line. `None` when one of them is no
    /// [`KernelParameter`] or there are too many.
    pub fn parse(command_line: &str) -> Option<Self> {
        let words: Result<Vec<_>, _> = command_line
            .split_whitespace()
            .map(KernelParameter::new)
            .collect();

        Self::new(words.ok()?).ok()
    }

    pub fn iter(&self) -> impl Iterator<Item = &KernelParameter> {
        self.0.iter()
    }

    pub fn contains(&self, parameter: &str) -> bool {
        self.0.iter().any(|own| own.as_str() == parameter)
    }

    /// With `parameter` at the end, unless it is there already.
    ///
    /// # Errors
    ///
    /// [`InvalidKernelParameters`] when the list is full.
    pub fn with(&self, parameter: KernelParameter) -> Result<Self, InvalidKernelParameters> {
        if self.contains(parameter.as_str()) {
            return Ok(self.clone());
        }

        let mut parameters = self.0.clone();

        parameters.push(parameter);
        Self::new(parameters)
    }

    /// Without every `parameter`.
    #[must_use = "returns a changed copy, the list itself stays as it was"]
    pub fn without(&self, parameter: &str) -> Self {
        Self(
            self.0
                .iter()
                .filter(|own| own.as_str() != parameter)
                .cloned()
                .collect(),
        )
    }
}

impl fmt::Display for KernelParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, parameter) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(" ")?;
            }

            f.write_str(parameter.as_str())?;
        }

        Ok(())
    }
}

impl TryFrom<Vec<KernelParameter>> for KernelParameters {
    type Error = InvalidKernelParameters;

    fn try_from(parameters: Vec<KernelParameter>) -> Result<Self, Self::Error> {
        Self::new(parameters)
    }
}

impl From<KernelParameters> for Vec<KernelParameter> {
    fn from(parameters: KernelParameters) -> Self {
        parameters.0
    }
}
