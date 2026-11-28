//! The fetch of a review: one `paru -G` for every base.

use std::path::Path;
use std::process::Command;

use toolbox_plan::protocol::Outcome;
use toolbox_plan::{AUR_CLONE, PackageName, StepId, TargetList};
use toolbox_text::OutsideText;

use super::{Console, FETCH_CAP, FETCH_DEADLINE};
use crate::apply::EventSink;
use crate::capture::{CaptureError, Policy};
use crate::report::{failure, output};

/// `paru -G` of every base, captured. Its lines name AUR packages and
/// versions, which is outside text, so they are cleaned before they show.
pub(super) fn fetch(
    console: &mut dyn Console,
    (paru, home): (&Path, &Path),
    bases: &TargetList<PackageName>,
    step: StepId,
    sink: &mut dyn EventSink,
) -> Result<(), Outcome> {
    let mut command = Command::new(paru);
    command
        .current_dir(home.join(AUR_CLONE))
        .args(["-G", "--"])
        .args(bases.iter().map(PackageName::as_str));

    let policy = Policy::tail(FETCH_CAP).capped().within(FETCH_DEADLINE);
    let fetched = match console.watched(command, policy) {
        Ok(fetched) => fetched,
        Err(CaptureError::Cancelled) => return Err(Outcome::Cancelled),
        Err(CaptureError::TooLarge) => {
            return Err(failure(
                step,
                sink,
                &format!("paru printed more than {FETCH_CAP} bytes, so the fetch is stopped"),
            ));
        }
        Err(error) => return Err(failure(step, sink, &format!("paru: {error}"))),
    };

    output(
        step,
        sink,
        &OutsideText::keep_lines(&fetched.output).into_string(),
    );
    output(
        step,
        sink,
        &OutsideText::keep_lines(&fetched.errors).into_string(),
    );

    if fetched.code != Some(0_i32) {
        return Err(failure(
            step,
            sink,
            &format!("paru exited with {:?}", fetched.code),
        ));
    }

    Ok(())
}
