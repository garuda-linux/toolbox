//! The words for a unit's states. Codes come from systemd and from
//! [`sub_code`], so a systemd word never reaches the page.

use std::borrow::Cow;

use egui::Context;
use toolbox_core::Toggle;
use toolbox_core::features::services::{ActiveState, FileState, UnitInfo, sub_code};
use toolbox_gui_kit::widgets::Tone;
use toolbox_plan::UnitOperation;

use crate::i18n;

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
    if unit.active == ActiveState::Active && sub == "exited" {
        return (i18n::text(ctx, "services-finished"), tone);
    }

    let state = i18n::text(ctx, &format!("services-active-{}", unit.active.as_str()));
    let text = if sub.is_empty() || sub == "other" {
        state
    } else {
        let detail = i18n::text(ctx, &format!("services-sub-{sub}"));
        i18n::text_with(
            ctx,
            "services-state-detail",
            &[("state", &state), ("detail", &detail)],
        )
    };

    (text, tone)
}

/// The file state worth a tag, with what it means. Enabled and disabled
/// show in the switch instead.
pub fn file(ctx: &Context, file: FileState) -> Option<(Cow<'static, str>, Cow<'static, str>)> {
    if matches!(file, FileState::Enabled | FileState::Disabled) {
        return None;
    }

    let id = format!("services-file-{}", file.as_str());
    Some((i18n::text(ctx, &id), i18n::text(ctx, &format!("{id}-hint"))))
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
