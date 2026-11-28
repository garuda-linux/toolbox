//! The window's words for a failed collection. Each [`Failure`] picks its
//! sentence by a literal id here, the parts go in as arguments, and the
//! text a program printed stays data.

#[cfg(test)]
mod tests;

use egui::Context;
use toolbox_core::features::diagnostics::{AnswerFault, Failure, HelperFault, Refusal};

use crate::i18n::{self, Arg};

/// What a failed collection says, in the window's language.
pub(super) fn text(ctx: &Context, failure: &Failure) -> String {
    match failure {
        Failure::NoProgramRan => plain(ctx, "diagnostics-failed-no-program"),
        Failure::ExitCode(code) => fill(
            ctx,
            "diagnostics-failed-exit-code",
            &[("code", Arg::from(code))],
        ),
        Failure::Signal => plain(ctx, "diagnostics-failed-signal"),
        Failure::Stderr(text) => text.to_owned(),
        Failure::Start { program, error } => fill(
            ctx,
            "diagnostics-failed-start",
            &[("program", Arg::from(program)), ("error", Arg::from(error))],
        ),
        Failure::TimedOut { program, seconds } => fill(
            ctx,
            "diagnostics-failed-timed-out",
            &[
                ("program", Arg::from(program)),
                ("seconds", Arg::from(seconds)),
            ],
        ),
        Failure::TooLarge { program } => fill(
            ctx,
            "diagnostics-failed-too-large",
            &[("program", Arg::from(program))],
        ),
        Failure::Read { path, error } => fill(
            ctx,
            "diagnostics-failed-read",
            &[("path", Arg::from(path)), ("error", Arg::from(error))],
        ),
        Failure::Helper(fault) => helper(ctx, fault),
        Failure::Answer(fault) => {
            let fault = answer(ctx, fault);
            fill(
                ctx,
                "diagnostics-failed-answer",
                &[("fault", Arg::from(&fault))],
            )
        }
        Failure::NotInstalled => plain(ctx, "diagnostics-failed-not-installed"),
        Failure::Refused(refusal) => plain(ctx, refused_id(*refusal)),
    }
}

const fn refused_id(refusal: Refusal) -> &'static str {
    match refusal {
        Refusal::Dismissed => "diagnostics-failed-dismissed",
        Refusal::Denied => "diagnostics-failed-denied",
        Refusal::NoAgent => "diagnostics-failed-no-agent",
        Refusal::HelperMissing => "diagnostics-failed-helper-not-installed",
    }
}

fn helper(ctx: &Context, fault: &HelperFault) -> String {
    match fault {
        HelperFault::Missing => plain(ctx, "diagnostics-failed-helper-missing"),
        HelperFault::Start { error } => fill(
            ctx,
            "diagnostics-failed-helper-start",
            &[("error", Arg::from(error))],
        ),
        HelperFault::TimedOut => plain(ctx, "diagnostics-failed-helper-timed-out"),
        HelperFault::TooLong => plain(ctx, "diagnostics-failed-helper-too-long"),
        HelperFault::Pkexec { words } => fill(
            ctx,
            "diagnostics-failed-helper-pkexec",
            &[("words", Arg::from(words))],
        ),
        HelperFault::Ended { code, words } => {
            let ending = code.map_or_else(
                || plain(ctx, "diagnostics-failed-ending-signal"),
                |code| {
                    fill(
                        ctx,
                        "diagnostics-failed-ending-code",
                        &[("code", Arg::from(code))],
                    )
                },
            );

            fill(
                ctx,
                "diagnostics-failed-helper-ended",
                &[("ending", Arg::from(&ending)), ("words", Arg::from(words))],
            )
        }
        HelperFault::Unreadable => plain(ctx, "diagnostics-failed-helper-unreadable"),
        HelperFault::Stopped => plain(ctx, "diagnostics-failed-helper-stopped"),
    }
}

fn answer(ctx: &Context, fault: &AnswerFault) -> String {
    match fault {
        AnswerFault::TooLong { length, limit } => fill(
            ctx,
            "diagnostics-failed-answer-too-long",
            &[("length", Arg::from(length)), ("limit", Arg::from(limit))],
        ),
        AnswerFault::Json(text) => text.to_owned(),
    }
}

fn plain(ctx: &Context, id: &str) -> String {
    i18n::text(ctx, id).into_owned()
}

fn fill(ctx: &Context, id: &str, args: &[(&'static str, Arg<'_>)]) -> String {
    i18n::text_with(ctx, id, args).into_owned()
}
