//! Build orders answered from captured `paru -Po --aur` output, for the
//! tests of other crates.

use std::sync::atomic::{AtomicUsize, Ordering};

use toolbox_plan::PackageName;
use toolbox_state::aur::order::{AurOrder, parse_order};

use crate::AurOrderSource;
use crate::error::ProbeError;

/// Each package's answer: the stdout paru printed for it, or the reason
/// it failed. Packages without an answer fail.
#[derive(Debug, Default)]
pub struct FakeAurOrders {
    answers: Vec<(&'static str, Result<&'static str, &'static str>)>,
    asked: AtomicUsize,
}

impl FakeAurOrders {
    pub const fn new(answers: Vec<(&'static str, Result<&'static str, &'static str>)>) -> Self {
        Self {
            answers,
            asked: AtomicUsize::new(0),
        }
    }

    /// How many times `order` was asked.
    pub fn asked(&self) -> usize {
        self.asked.load(Ordering::SeqCst)
    }
}

impl AurOrderSource for FakeAurOrders {
    fn order(&self, package: &PackageName) -> Result<AurOrder, ProbeError> {
        self.asked.fetch_add(1, Ordering::SeqCst);

        let answer = self
            .answers
            .iter()
            .find(|(name, _)| *name == package.as_str())
            .map(|(_, answer)| *answer);

        match answer {
            Some(Ok(text)) => parse_order(text).map_err(|error| ProbeError::Invalid {
                what: format!("the build order of {package}"),
                error: error.into(),
            }),
            Some(Err(reason)) => Err(ProbeError::fake(reason)),
            None => Err(ProbeError::fake(format!("no answer for {package}"))),
        }
    }
}
