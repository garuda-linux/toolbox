//! One root segment, run through the helper session of its plan.

use toolbox_plan::protocol::{Mode, Request};
use toolbox_plan::{RootAction, Segment, Step};

use super::session::Session;
use super::{Launcher, Run};

/// The request of a whole plan: its root steps in plan order, and a gate
/// before the first step of each root segment after the first.
pub fn request(segments: &[Segment], mode: Mode) -> Request {
    let mut steps = Vec::new();
    let mut gates = Vec::new();
    for segment in segments {
        if let Segment::Root(root) = segment {
            if let Some(first) = root.first()
                && !steps.is_empty()
            {
                gates.push(first.id);
            }

            steps.extend(root.iter().cloned());
        }
    }

    let mut request = Request::new(mode, steps);
    request.gates = gates;
    request
}

/// One root segment. The plan's first one starts the helper session, a
/// later one passes its gate first.
pub async fn run(
    steps: &[Step<RootAction>],
    request: &Request,
    session: &mut Option<Session>,
    run: &mut Run<'_>,
    launcher: &dyn Launcher,
) -> bool {
    let (Some(first), Some(last)) = (steps.first(), steps.last()) else {
        return true;
    };

    let gated = session.is_some();
    if !gated {
        *session = Session::start(request, run, launcher).await;
    }

    let Some(open) = session.as_mut() else {
        return false;
    };

    if gated && !open.proceed(first.id, run, launcher).await {
        return false;
    }

    open.segment(last.id, run, launcher).await
}
