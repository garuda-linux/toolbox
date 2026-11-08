use std::time::SystemTime;

use serde::Serialize;
use toolbox_cli::output::size;
use toolbox_core::features::status::{cleanup_due, disk_low};
use toolbox_system::Facts;

use super::{Part, Shown, counted, counted_part, joined_or_none, part, row};

#[cfg(test)]
mod tests;

const SECONDS_PER_HOUR: u64 = 3_600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Packages {
    pub installed: usize,
    /// In no repository, `null` without a synced database.
    pub foreign: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Disk {
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub low: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FactsView {
    pub packages: Part<Packages>,
    pub kernel: Part<String>,
    pub uptime_hours: Part<u64>,
    pub disk: Part<Disk>,
    pub failed_services: Part<Vec<String>>,
    pub orphans: Part<Vec<String>>,
    pub package_cache_bytes: Part<u64>,
}

pub fn facts_view(facts: &Facts, now: SystemTime) -> FactsView {
    FactsView {
        packages: part(&facts.packages, |count| Packages {
            installed: count.installed,
            foreign: count.foreign,
        }),
        kernel: part(&facts.kernel, Clone::clone),
        uptime_hours: part(&facts.booted, |booted| {
            now.duration_since(*booted)
                .map_or(0, |up| up.as_secs() / SECONDS_PER_HOUR)
        }),
        disk: part(&facts.disk, |disk| Disk {
            total_bytes: disk.total,
            free_bytes: disk.free,
            low: disk_low(disk.free, disk.total),
        }),
        failed_services: part(&facts.failed_units, |units| {
            units.iter().map(|unit| unit.as_str().to_owned()).collect()
        }),
        orphans: part(&facts.orphans, |orphans| {
            orphans
                .iter()
                .map(|package| package.as_str().to_owned())
                .collect()
        }),
        package_cache_bytes: part(&facts.package_cache, |bytes| *bytes),
    }
}

pub(super) fn rows(view: &FactsView) -> [(&'static str, Shown); 7] {
    [
        row("Packages", &view.packages, |count| {
            count.foreign.map_or_else(
                || count.installed.to_string(),
                |foreign| format!("{} ({foreign} foreign)", count.installed),
            )
        }),
        row("Kernel", &view.kernel, Clone::clone),
        row("Uptime", &view.uptime_hours, |hours| match hours / 24 {
            0 => format!("{hours} h"),
            1 => "1 day".to_owned(),
            days => format!("{days} days"),
        }),
        row("Disk", &view.disk, |disk| {
            let low = if disk.low { ", low" } else { "" };
            format!(
                "{} free of {}{low}",
                size(disk.free_bytes),
                size(disk.total_bytes)
            )
        }),
        row("Failed services", &view.failed_services, |units| {
            joined_or_none(units)
        }),
        row("Orphans", &view.orphans, |orphans| joined_or_none(orphans)),
        row("Package cache", &view.package_cache_bytes, |bytes| {
            size(*bytes)
        }),
    ]
}

pub(super) fn attention(view: &FactsView) -> Vec<String> {
    let low = matches!(&view.disk, Part::Ok { value } if value.low);
    let failed = counted_part(&view.failed_services, Vec::len);
    let orphans = counted_part(&view.orphans, Vec::len);
    let cache = match view.package_cache_bytes {
        Part::Ok { value } => value,
        Part::Unavailable | Part::Failed { cause: _ } => 0,
    };

    [
        low.then(|| "disk space low".to_owned()),
        counted(failed, "failed service", "failed services"),
        cleanup_due(orphans, cache).then(|| "clean-up due".to_owned()),
    ]
    .into_iter()
    .flatten()
    .collect()
}
