//! The Chaotic-AUR mirrors of a dry run: `toolbox_plan::SAMPLE_MIRRORS`.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use toolbox_plan::{Mirrorlist, SAMPLE_MIRRORS};
use toolbox_system::mirrors::{MirrorRanking, RankProblem};

/// Answers the placeholder list at once, for a dry run:
/// `rate-mirrors` would run for minutes and ask every mirror.
#[derive(Clone, Copy, Debug)]
pub struct SampleMirrors;

impl MirrorRanking for SampleMirrors {
    fn rank(&self, _cancel: &Arc<AtomicBool>) -> Result<Mirrorlist, RankProblem> {
        Mirrorlist::new(SAMPLE_MIRRORS).map_err(|error| RankProblem::Invalid(error.to_string()))
    }
}
