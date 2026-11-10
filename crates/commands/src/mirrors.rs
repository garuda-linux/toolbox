//! `maintenance run rank-mirrors`: the Chaotic-AUR mirrors ranked by
//! `rate-mirrors` as the user, then the list installed as root, as the
//! Maintenance page's tool does. The tool is installed first when it is
//! missing.

#[cfg(test)]
mod tests;

use std::path::Path;

use tokio::runtime::Runtime;
use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_core::features::maintenance::RATE_MIRRORS_PACKAGE;
use toolbox_core::features::maintenance::page::{RATE_MIRRORS_PROGRAM, install};
use toolbox_plan::{Plan, RootAction};
use toolbox_system::mirrors::{MirrorRanking, RateMirrors};

use crate::act::run;
use crate::out::{Out, failure};
use crate::spinner;

/// Its id in `maintenance list` and `maintenance run`.
pub const RANK_MIRRORS: &str = "rank-mirrors";

const RANKING: &str = "Ranking the Chaotic-AUR mirrors, this takes a few minutes…";

pub fn rank(confirm: Confirm, runtime: &Runtime, out: &mut Out<'_>) -> Exit {
    let installed = Path::new(RATE_MIRRORS_PROGRAM).exists();

    steps(
        installed,
        confirm,
        &RateMirrors::system(),
        out,
        |plan, out| run(Ok(plan), confirm, None, runtime, out),
    )
}

/// `carry_out` runs each plan as `run` does, a stand-in in tests.
fn steps(
    installed: bool,
    confirm: Confirm,
    ranking: &dyn MirrorRanking,
    out: &mut Out<'_>,
    mut carry_out: impl FnMut(Plan, &mut Out<'_>) -> Exit,
) -> Exit {
    if !installed {
        let exit = carry_out(install(&RATE_MIRRORS_PACKAGE), out);
        if exit != Exit::Success || confirm.dry_run {
            return exit;
        }
    }

    let quiet = confirm.json || out.quiet();
    match spinner::while_working(quiet, RANKING, || ranking.rank()) {
        Ok(mirrorlist) => carry_out(
            Plan::root(RootAction::InstallMirrorlist { mirrorlist }),
            out,
        ),
        Err(problem) => {
            failure(
                out,
                "the Chaotic-AUR mirrors could not be ranked",
                &problem.to_string(),
            );
            Exit::Failure
        }
    }
}
