use std::io;
use std::path::Path;

use toolbox_plan::procedure::CHAOTIC_MIRRORLIST;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{Mirrorlist, StepId};

use crate::apply::EventSink;
use crate::report::{failure, note};
use crate::secure;

/// Writes the Chaotic-AUR mirrorlist below `root` and keeps the old one as
/// `chaotic-mirrorlist.bak`. Returns whether there was an old one.
///
/// # Errors
///
/// When `etc/pacman.d` below `root` is missing or reached through a
/// symlink, or the write fails.
pub(super) fn install_mirrorlist(root: &Path, mirrorlist: &Mirrorlist) -> io::Result<bool> {
    let (dir, name) = secure::open_parent(root, CHAOTIC_MIRRORLIST)?;
    let backup = format!("{name}.bak");

    secure::replace_system_file(&dir, name, mirrorlist.as_str().as_bytes(), Some(&backup))
}

pub(super) fn mirrorlist_written(
    root: &Path,
    step: StepId,
    mirrorlist: &Mirrorlist,
    sink: &mut dyn EventSink,
) -> Outcome {
    match install_mirrorlist(root, mirrorlist) {
        Ok(true) => note(
            step,
            sink,
            "wrote the Chaotic-AUR mirrorlist, the old one is chaotic-mirrorlist.bak",
            Outcome::Succeeded,
        ),
        Ok(false) => note(
            step,
            sink,
            "wrote the Chaotic-AUR mirrorlist",
            Outcome::Succeeded,
        ),
        Err(error) => failure(
            step,
            sink,
            &format!("cannot write the Chaotic-AUR mirrorlist: {error}"),
        ),
    }
}
