//! The words of the Boot Repair page that the page and its dialog share.

use egui::Context;
use toolbox_cli::output::size;
use toolbox_core::features::bootrepair::page::{Progress, TargetChoice};
use toolbox_core::features::bootrepair::{Encryption, Firmware, System};
use toolbox_plan::{Name, RootLayout};

use crate::i18n;

/// "/dev/sda2 · Garuda · 200.0 GiB · btrfs", with the parts that are known.
pub fn system_line(system: &System) -> String {
    line([
        Some(system.device().to_owned()),
        Some(system.label.clone()),
        (system.size > 0).then(|| size(system.size)),
        Some(system.file_system.clone()),
    ])
}

/// The same for a place the boot loader can go.
pub fn target_line(choice: &TargetChoice) -> String {
    let device = choice.target.device();

    line([
        Some(device.to_owned()),
        Some(choice.label.clone()),
        (choice.size > 0).then(|| size(choice.size)),
    ])
}

fn line(parts: impl IntoIterator<Item = Option<String>>) -> String {
    parts
        .into_iter()
        .flatten()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

/// "/dev/sda, /dev/sdb": the disks the system lies on.
pub fn disks(system: &System) -> String {
    let disks: Vec<&str> = system.disks.iter().map(Name::as_str).collect();
    disks.join(", ")
}

pub fn firmware(ctx: &Context, firmware: Firmware) -> (String, String) {
    let (name, detail) = match firmware {
        Firmware::Uefi => ("repair-uefi", "repair-uefi-detail"),
        Firmware::Bios => ("repair-bios", "repair-bios-detail"),
    };

    (
        i18n::text(ctx, name).into_owned(),
        i18n::text(ctx, detail).into_owned(),
    )
}

pub fn encryption(ctx: &Context, encryption: Encryption) -> String {
    let id = match encryption {
        Encryption::None => "repair-encryption-none",
        Encryption::Locked => "repair-encryption-locked",
        Encryption::Unlocked => "repair-encryption-unlocked",
    };

    i18n::text(ctx, id).into_owned()
}

pub fn layout(ctx: &Context, layout: RootLayout) -> String {
    i18n::text(ctx, &format!("repair-layout-{}", layout.as_str())).into_owned()
}

/// How the repair this page started went, `None` without one.
pub fn progress(ctx: &Context, progress: Progress) -> Option<String> {
    let id = match progress {
        Progress::None => return None,
        Progress::Waiting | Progress::Running => "repair-running",
        Progress::Succeeded => "repair-succeeded",
        Progress::Failed => "repair-failed-run",
        Progress::Busy => "repair-busy",
    };

    Some(i18n::text(ctx, id).into_owned())
}

/// Why a repair did not start.
pub fn refusal(ctx: &Context, code: &str) -> String {
    let known = [
        "not-offered",
        "unavailable",
        "unknown-system",
        "in-use",
        "unknown-target",
        "unknown-layout",
        "empty",
        "too-long",
        "control-character",
    ];

    let code = if known.contains(&code) { code } else { "other" };
    i18n::text(ctx, &format!("repair-refused-{code}")).into_owned()
}

/// The layout as the choice on the page names it.
pub fn layout_short(ctx: &Context, layout: RootLayout) -> String {
    i18n::text(ctx, &format!("repair-layout-{}-short", layout.as_str())).into_owned()
}
