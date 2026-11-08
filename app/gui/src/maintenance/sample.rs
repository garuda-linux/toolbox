//! The Chaotic-AUR mirrors of a self-test build's dry run. Release builds
//! leave them out.

use toolbox_plan::Mirrorlist;
use toolbox_system::mirrors::{MirrorRanking, RankProblem};

/// The self-test's Chaotic-AUR list. It never reaches a mirror.
const SAMPLE_MIRRORS: &str = "# Chaotic-AUR mirrors of the self-test, never ranked\n\
    Server = https://cdn-mirror.chaotic.cx/$repo/$arch\n\
    Server = https://geo-mirror.chaotic.cx/$repo/$arch\n";

/// Answers a fixed list at once, for the self-test:
/// `rate-mirrors` would run for minutes and ask every mirror.
#[derive(Clone, Copy, Debug)]
pub struct SampleMirrors;

impl MirrorRanking for SampleMirrors {
    fn rank(&self) -> Result<Mirrorlist, RankProblem> {
        Mirrorlist::new(SAMPLE_MIRRORS).map_err(|error| RankProblem::Invalid(error.to_string()))
    }
}
