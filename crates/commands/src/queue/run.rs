//! `queue apply`: the plan of the queue, carried out and settled, as the
//! window's Apply does it.

use toolbox_cli::Exit;
use toolbox_cli::args::Confirm;
use toolbox_core::planned;
use toolbox_plan::{Plan, PlannedAction};
use toolbox_system::{Paru, RouteSource as _, SystemRoute};

use super::said::say;
use super::{Context, Failed, ready, settle, snapshot};
use crate::out::Out;
use crate::terminal::Terminal;
use crate::{apply, root};

/// The window's queue belongs to the window, so a change never runs twice.
pub(super) fn apply_queue(confirm: Confirm, context: &Context<'_>, out: &mut Out<'_>) -> Exit {
    let Context {
        store,
        catalogs,
        system,
        runtime,
        aur,
    } = *context;

    if store.in_window() {
        say(
            out,
            "the window holds the queue, so only the window applies it",
            "apply it in the window, or close the window",
        );
        return Exit::Unavailable;
    }

    let saved = match store.saved() {
        Ok(saved) => saved,
        Err(problem) => return Failed::Store(problem).say(out),
    };

    let reads = ready::HelperReads {
        settings: out.settings(),
        aur,
    };

    let mut changes = match ready::checked(&saved, system, &reads) {
        Ok(changes) => changes,
        Err(refusal) => return ready::refused(refusal, out),
    };

    let route = SystemRoute::system().route();
    let sudo = root::through_sudo(out.settings());
    if let Err(refused) = ready::prepared(&mut changes, system, sudo, route, &Paru::default()) {
        return Failed::Refused(refused).say(out);
    }

    let (planned, owners) = planned(&changes);
    let ended = apply::carry_out_steps(
        planned.clone(),
        confirm,
        None,
        runtime.handle(),
        root::launcher(out.settings()),
        &mut Terminal {
            color: out.settings().color,
        },
        out,
    );

    let exit = ended.exit;
    let succeeded = succeeded_actions(&planned, &ended);

    let ran = !confirm.dry_run && !matches!(exit, Exit::Declined | Exit::NeedsYes | Exit::Usage);
    if ran
        && let Err(failed) = settle(
            store,
            &ready::after(snapshot(catalogs), &changes, &reads),
            &owners,
            |step| ended.ended_well(step),
            &succeeded,
        )
    {
        failed.say(out);
    }

    exit
}

/// The actions of the steps of `planned` that `ended` says succeeded.
fn succeeded_actions(planned: &Plan, ended: &apply::Ended) -> Vec<PlannedAction> {
    planned
        .steps()
        .iter()
        .filter(|step| ended.succeeded.contains(&step.id))
        .map(|step| step.action.clone())
        .collect()
}
