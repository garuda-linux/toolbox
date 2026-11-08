#[cfg(test)]
mod tests;

/// In `bootctl`'s order.
const ESP_PLACES: [&str; 3] = ["efi", "boot", "boot/efi"];

pub fn esp_place(mountinfo: &str) -> Option<&'static str> {
    let fat: Vec<&str> = mountinfo
        .lines()
        .filter_map(|line| {
            let (mount, filesystem) = line.split_once(" - ")?;
            let place = mount.split_whitespace().nth(4)?;
            let kind = filesystem.split_whitespace().next()?;

            (kind == "vfat").then_some(place)
        })
        .collect();

    ESP_PLACES.into_iter().find(|place| {
        fat.iter()
            .any(|mounted| mounted.strip_prefix('/') == Some(*place))
    })
}
