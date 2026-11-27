//! The words for a unit's states. Codes come from systemd and from
//! [`sub_code`], so a systemd word never reaches the page.

use std::borrow::Cow;

use egui::Context;
use toolbox_core::Toggle;
use toolbox_core::features::services::{
    ActiveState, FileState, Shown, SubCode, UnitInfo, sub_code,
};
use toolbox_gui_kit::widgets::Tone;
use toolbox_plan::UnitOperation;

use crate::i18n;
use crate::i18n::Arg;

pub fn state(ctx: &Context, unit: &UnitInfo) -> (Cow<'static, str>, Tone) {
    let tone = match unit.active {
        ActiveState::Active => Tone::Success,
        ActiveState::Failed => Tone::Error,
        ActiveState::Inactive => Tone::Neutral,
        ActiveState::Activating
        | ActiveState::Deactivating
        | ActiveState::Reloading
        | ActiveState::Maintenance
        | ActiveState::Refreshing
        | ActiveState::Other => Tone::Warning,
    };

    let sub = sub_code(unit.active, &unit.sub_state);

    // "Running (exited)" reads as a contradiction for a one-shot job that
    // did its work and stays active.
    if unit.active == ActiveState::Active && sub == Some(SubCode::Exited) {
        return (i18n::text(ctx, "services-finished"), tone);
    }

    let state = i18n::text(ctx, active_id(unit.active));
    let text = match sub.and_then(sub_detail_id) {
        None => state,
        Some(id) => {
            let detail = i18n::text(ctx, id);
            i18n::text_with(
                ctx,
                "services-state-detail",
                &[("state", Arg::from(&state)), ("detail", Arg::from(&detail))],
            )
        }
    };

    (text, tone)
}

/// The ids are literals, so grep and the catalog checks see each one. A
/// sub state newer than the app has no words of its own.
const fn sub_detail_id(sub: SubCode) -> Option<&'static str> {
    match sub {
        SubCode::Running => Some("services-sub-running"),
        SubCode::Exited => Some("services-sub-exited"),
        SubCode::Listening => Some("services-sub-listening"),
        SubCode::Waiting => Some("services-sub-waiting"),
        SubCode::Active => Some("services-sub-active"),
        SubCode::Mounted => Some("services-sub-mounted"),
        SubCode::Plugged => Some("services-sub-plugged"),
        SubCode::Elapsed => Some("services-sub-elapsed"),
        SubCode::Abandoned => Some("services-sub-abandoned"),
        SubCode::Failed => Some("services-sub-failed"),
        SubCode::Cleaning => Some("services-sub-cleaning"),
        SubCode::Dead => Some("services-sub-dead"),
        SubCode::Restarting => Some("services-sub-restarting"),
        SubCode::Starting => Some("services-sub-starting"),
        SubCode::Stopping => Some("services-sub-stopping"),
        SubCode::Reloading => Some("services-sub-reloading"),
        SubCode::Other => None,
    }
}

/// The file state worth a tag, with what it means. Enabled and disabled
/// show in the switch instead.
pub fn file(ctx: &Context, file: FileState) -> Option<(Cow<'static, str>, Cow<'static, str>)> {
    if matches!(file, FileState::Enabled | FileState::Disabled) {
        return None;
    }

    let (name, hint) = match file {
        FileState::Enabled | FileState::Disabled => return None,
        FileState::Static => ("services-file-static", "services-file-static-hint"),
        FileState::Masked => ("services-file-masked", "services-file-masked-hint"),
        FileState::Indirect => ("services-file-indirect", "services-file-indirect-hint"),
        FileState::Generated => ("services-file-generated", "services-file-generated-hint"),
        FileState::Linked => ("services-file-linked", "services-file-linked-hint"),
        FileState::Alias => ("services-file-alias", "services-file-alias-hint"),
        FileState::Transient => ("services-file-transient", "services-file-transient-hint"),
        FileState::Bad => ("services-file-bad", "services-file-bad-hint"),
        FileState::Other => ("services-file-other", "services-file-other-hint"),
        FileState::Unknown => ("services-file-unknown", "services-file-unknown-hint"),
    };

    Some((i18n::text(ctx, name), i18n::text(ctx, hint)))
}

/// The ids are literals, so grep and the catalog checks see each one.
pub const fn show_id(shown: Shown) -> &'static str {
    match shown {
        Shown::Running => "services-show-running",
        Shown::Enabled => "services-show-enabled",
        Shown::Disabled => "services-show-disabled",
        Shown::Masked => "services-show-masked",
        Shown::Dead => "services-show-dead",
        Shown::Failed => "services-show-failed",
        Shown::All => "services-show-all",
    }
}

const fn active_id(active: ActiveState) -> &'static str {
    match active {
        ActiveState::Active => "services-active-active",
        ActiveState::Reloading => "services-active-reloading",
        ActiveState::Inactive => "services-active-inactive",
        ActiveState::Failed => "services-active-failed",
        ActiveState::Activating => "services-active-activating",
        ActiveState::Deactivating => "services-active-deactivating",
        ActiveState::Maintenance => "services-active-maintenance",
        ActiveState::Refreshing => "services-active-refreshing",
        ActiveState::Other => "services-active-other",
    }
}

pub fn queued(
    ctx: &Context,
    enable: Option<Toggle>,
    mask: Option<Toggle>,
) -> Vec<Cow<'static, str>> {
    let enable = enable.map(|wish| {
        if wish.is_on() {
            "services-enables"
        } else {
            "services-disables"
        }
    });

    let mask = mask.map(|wish| {
        if wish.is_on() {
            "services-masks"
        } else {
            "services-unmasks"
        }
    });

    [enable, mask]
        .into_iter()
        .flatten()
        .map(|id| i18n::text(ctx, id))
        .collect()
}

pub const fn operation(operation: UnitOperation) -> &'static str {
    match operation {
        UnitOperation::Start => "services-start",
        UnitOperation::Stop => "services-stop",
        UnitOperation::Restart => "services-restart",
        UnitOperation::Reload => "services-reload",
    }
}
