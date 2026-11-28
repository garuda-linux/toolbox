//! The window's words for a failed probe. Each [`ProbeError`] picks its
//! sentence by a literal id here, the parts go in as arguments, and the
//! text the system printed stays data.

#[cfg(test)]
mod tests;

use std::path::Path;
use std::process::ExitStatus;
use std::time::Duration;

use egui::Context;
use toolbox_plan::{NameKind, Reason};
use toolbox_system::{InvalidCause, MissingFact, ProbeError, Stop, Subject};

use crate::i18n::{self, Arg};

/// What a failed probe says, `None` when the source is not there.
pub fn failure(ctx: &Context, error: &ProbeError) -> Option<String> {
    let text = match error {
        ProbeError::Unavailable => return None,
        ProbeError::Command {
            command_line,
            status,
            stderr,
        } => {
            let ended = ended(ctx, *status);
            fill(
                ctx,
                "probe-cause-command",
                &[
                    ("command", Arg::from(command_line)),
                    ("status", Arg::from(&ended)),
                    ("stderr", Arg::from(stderr)),
                ],
            )
        }
        ProbeError::TimedOut {
            command_line,
            after,
        } => timed_out(ctx, command_line, *after),
        ProbeError::Stopped { command_line, why } => {
            let id = match why {
                Stop::Cancelled => "probe-cause-cancelled",
                Stop::TooLarge => "probe-cause-too-large",
            };

            fill(ctx, id, &[("command", Arg::from(command_line))])
        }
        ProbeError::Json { what, error: cause } => what_error(ctx, what, &cause.to_string()),
        ProbeError::Invalid { what, error: value } => {
            let cause = invalid(ctx, &value.cause());
            what_error(ctx, what, &cause)
        }
        ProbeError::Alpm { what, error: cause } => what_error(ctx, what, &cause.to_string()),
        ProbeError::Syscall { what, error: cause } => what_error(ctx, what, &cause.to_string()),
        ProbeError::Io { what, error: cause } => what_error(ctx, what, &cause.to_string()),
        ProbeError::NotUtf8 { what, raw } => {
            let what = subject(ctx, what);
            let raw = raw.display().to_string();
            fill(
                ctx,
                "probe-cause-not-utf8",
                &[("what", Arg::from(&what)), ("raw", Arg::from(&raw))],
            )
        }
        ProbeError::Missing(fact) => missing(ctx, fact),
        ProbeError::NotAFile { path } => path_words(ctx, "probe-cause-not-a-file", path),
        ProbeError::NulInPath { path } => path_words(ctx, "probe-cause-nul-in-path", path),
        ProbeError::Panicked => i18n::text(ctx, "probe-cause-panicked").into_owned(),
    };

    Some(text)
}

fn fill(ctx: &Context, id: &str, args: &[(&'static str, Arg<'_>)]) -> String {
    i18n::text_with(ctx, id, args).into_owned()
}

fn timed_out(ctx: &Context, command_line: &str, after: Duration) -> String {
    let seconds = after.as_secs_f32().to_string();
    fill(
        ctx,
        "probe-cause-timed-out",
        &[
            ("command", Arg::from(command_line)),
            ("seconds", Arg::from(&seconds)),
        ],
    )
}

fn ended(ctx: &Context, status: ExitStatus) -> String {
    status.code().map_or_else(
        || i18n::text(ctx, "probe-cause-signal").into_owned(),
        |code| fill(ctx, "probe-cause-exit-code", &[("code", Arg::from(code))]),
    )
}

fn what_error(ctx: &Context, what: &Subject, error: &str) -> String {
    let what = subject(ctx, what);
    fill(
        ctx,
        "probe-cause-what",
        &[("what", Arg::from(&what)), ("error", Arg::from(error))],
    )
}

fn subject(ctx: &Context, what: &Subject) -> String {
    match what {
        Subject::Data(text) => text.to_owned(),
        Subject::LocalDatabase => i18n::text(ctx, "probe-subject-local-database").into_owned(),
        Subject::Disks => i18n::text(ctx, "probe-subject-disks").into_owned(),
        Subject::RunningKernel => i18n::text(ctx, "probe-subject-running-kernel").into_owned(),
        Subject::RouteList => i18n::text(ctx, "probe-subject-route-list").into_owned(),
        Subject::BuildOrder { package } => fill(
            ctx,
            "probe-subject-build-order",
            &[("package", Arg::from(package))],
        ),
        Subject::SampleBuildOrder { package } => fill(
            ctx,
            "probe-subject-sample-build-order",
            &[("package", Arg::from(package))],
        ),
    }
}

fn path_words(ctx: &Context, id: &str, path: &Path) -> String {
    let path = path.display().to_string();
    fill(ctx, id, &[("path", Arg::from(&path))])
}

fn missing(ctx: &Context, fact: &MissingFact) -> String {
    match fact {
        MissingFact::UserId { path } => path_words(ctx, "probe-cause-user-id", path),
        MissingFact::Root { path } => path_words(ctx, "probe-cause-root", path),
        MissingFact::BootTime => i18n::text(ctx, "probe-cause-boot-time").into_owned(),
        MissingFact::KernelRelease => i18n::text(ctx, "probe-cause-kernel-release").into_owned(),
    }
}

fn invalid(ctx: &Context, cause: &InvalidCause) -> String {
    match cause {
        InvalidCause::Name {
            kind,
            value,
            reason,
        } => {
            let kind = i18n::text(ctx, kind_id(*kind));
            let reason = reason_words(ctx, *reason);
            fill(
                ctx,
                "probe-cause-name",
                &[
                    ("kind", Arg::from(&kind)),
                    ("value", Arg::from(value)),
                    ("reason", Arg::from(&reason)),
                ],
            )
        }
        InvalidCause::Listing { words } => {
            fill(ctx, "probe-cause-listing", &[("words", Arg::from(words))])
        }
        InvalidCause::OrderLine { line } => {
            fill(ctx, "probe-cause-order-line", &[("line", Arg::from(line))])
        }
    }
}

pub(super) fn reason_words(ctx: &Context, reason: Reason) -> String {
    match reason {
        Reason::Empty => i18n::text(ctx, "name-reason-empty").into_owned(),
        Reason::TooLong { max } => fill(ctx, "name-reason-too-long", &[("max", Arg::from(max))]),
        Reason::InvalidStart(character) => {
            let character = character.to_string();
            fill(
                ctx,
                "name-reason-invalid-start",
                &[("character", Arg::from(&character))],
            )
        }
        Reason::InvalidCharacter(character) => {
            let character = character.escape_debug().to_string();
            fill(
                ctx,
                "name-reason-invalid-character",
                &[("character", Arg::from(&character))],
            )
        }
        Reason::UnknownUnitType => i18n::text(ctx, "name-reason-unknown-unit-type").into_owned(),
        Reason::InvalidComponent => i18n::text(ctx, "name-reason-invalid-component").into_owned(),
        Reason::WrongFileType => i18n::text(ctx, "name-reason-wrong-file-type").into_owned(),
        Reason::UnknownDevice => i18n::text(ctx, "name-reason-unknown-device").into_owned(),
    }
}

/// The ids are literals, so grep and the catalog checks see each one.
pub(super) const fn kind_id(kind: NameKind) -> &'static str {
    match kind {
        NameKind::Package => "name-kind-package",
        NameKind::Unit => "name-kind-unit",
        NameKind::User => "name-kind-user",
        NameKind::Group => "name-kind-group",
        NameKind::Locale => "name-kind-locale",
        NameKind::KernelModule => "name-kind-kernel-module",
        NameKind::Charset => "name-kind-charset",
        NameKind::DkmsModule => "name-kind-dkms-module",
        NameKind::DkmsVersion => "name-kind-dkms-version",
        NameKind::KernelRelease => "name-kind-kernel-release",
        NameKind::Host => "name-kind-host",
        NameKind::KernelParameter => "name-kind-kernel-parameter",
        NameKind::PlymouthTheme => "name-kind-plymouth-theme",
        NameKind::MenuEntry => "name-kind-menu-entry",
        NameKind::GrubPicture => "name-kind-grub-picture",
        NameKind::GrubTheme => "name-kind-grub-theme",
        NameKind::Disk => "name-kind-disk",
        NameKind::Partition => "name-kind-partition",
        NameKind::Volume => "name-kind-volume",
        NameKind::Mapper => "name-kind-mapper",
        NameKind::BootloaderId => "name-kind-bootloader-id",
        NameKind::LoaderEntry => "name-kind-loader-entry",
        NameKind::LoaderDefault => "name-kind-loader-default",
        NameKind::BuildRun => "name-kind-build-run",
    }
}
