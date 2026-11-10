#[cfg(test)]
mod tests;

use toolbox_plan::{DiskDevice, RootLayout, Toggle};

use super::{Firmware, RepairFacts, System, Target};

/// What the app runs on: the live ISO, where Boot Repair belongs, or an
/// installed system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Booted {
    Live,
    Installed,
}

impl Booted {
    pub const fn of_live(live: bool) -> Self {
        if live { Self::Live } else { Self::Installed }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Offer {
    /// Neither the live ISO nor the test flag.
    #[default]
    Hidden,
    Live,
    /// Shown by the flag on an installed system.
    Test,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Facts {
    #[default]
    NotAsked,
    Loading,
    Failed,
    Known(RepairFacts),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Progress {
    #[default]
    None,
    /// Sent to the apply service, which has not answered yet.
    Waiting,
    Running,
    Succeeded,
    Failed,
    /// Another run was going on, so this one never started.
    Busy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Hidden,
    Loading,
    Failed,
    Missing,
    Ready,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetChoice {
    pub target: Target,
    pub label: String,
    /// In bytes, 0 when not known.
    pub size: u64,
    /// The disk it lies on, or the disk itself.
    pub disk: DiskDevice,
}

impl Offer {
    /// The live ISO offers the page, `--boot-repair` (`forced`) too.
    pub const fn of(booted: Booted, forced: Toggle) -> Self {
        match (booted, forced) {
            (Booted::Live, _) => Self::Live,
            (Booted::Installed, Toggle::On) => Self::Test,
            (Booted::Installed, Toggle::Off) => Self::Hidden,
        }
    }
}

impl Facts {
    /// The facts once known, and only on a page that is offered.
    pub const fn known(&self, offer: Offer) -> Option<&RepairFacts> {
        match (offer, self) {
            (Offer::Hidden, _) | (_, Self::NotAsked | Self::Loading | Self::Failed) => None,
            (Offer::Live | Offer::Test, Self::Known(facts)) => Some(facts),
        }
    }
}

impl Progress {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Waiting | Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Busy => "busy",
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn started(self) -> Self {
        match self {
            Self::Waiting => Self::Running,
            other @ (Self::None | Self::Running | Self::Succeeded | Self::Failed | Self::Busy) => {
                other
            }
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn refused(self) -> Self {
        match self {
            Self::Waiting => Self::Busy,
            other @ (Self::None | Self::Running | Self::Succeeded | Self::Failed | Self::Busy) => {
                other
            }
        }
    }

    /// A plan ended. Only the one this page started has an outcome here.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn finished(self, succeeded: bool) -> Self {
        match (self, succeeded) {
            (Self::Running, true) => Self::Succeeded,
            (Self::Running, false) => Self::Failed,
            (other, _) => other,
        }
    }

    /// The outcome taken off the page. A run on its way stays.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub const fn forgotten(self) -> Self {
        match self {
            Self::Waiting | Self::Running => self,
            Self::None | Self::Succeeded | Self::Failed | Self::Busy => Self::None,
        }
    }
}

pub const fn state(offer: Offer, facts: &Facts) -> State {
    match (offer, facts) {
        (Offer::Hidden, _) => State::Hidden,
        (_, Facts::NotAsked | Facts::Loading) => State::Loading,
        (_, Facts::Failed) => State::Failed,
        (_, Facts::Known(known)) if !known.missing.is_empty() => State::Missing,
        (_, Facts::Known(_)) => State::Ready,
    }
}

pub fn system<'facts>(facts: &'facts RepairFacts, picked: Option<&str>) -> Option<&'facts System> {
    let systems = &facts.inventory.systems;

    picked
        .and_then(|device| facts.inventory.system(device))
        .or_else(|| systems.iter().find(|system| !system.in_use))
        .or_else(|| systems.first())
}

pub fn targets(facts: &RepairFacts) -> Vec<TargetChoice> {
    let inventory = &facts.inventory;
    match facts.firmware {
        Firmware::Uefi => inventory
            .esps
            .iter()
            .filter(|esp| !esp.in_use)
            .map(|esp| TargetChoice {
                target: Target::Esp(esp.device.clone()),
                label: esp.label.clone(),
                size: esp.size,
                disk: esp.disk.clone(),
            })
            .collect(),
        Firmware::Bios => inventory
            .disks
            .iter()
            .filter(|disk| !disk.in_use)
            .map(|disk| TargetChoice {
                target: Target::Disk(disk.device.clone()),
                label: disk.model.clone(),
                size: disk.size,
                disk: disk.device.clone(),
            })
            .collect(),
    }
}

fn usual_target(facts: &RepairFacts, system: &System) -> Option<Target> {
    match facts.firmware {
        Firmware::Uefi => facts
            .inventory
            .usual_esp(system)
            .map(|esp| Target::Esp(esp.device.clone())),
        Firmware::Bios => facts
            .inventory
            .usual_disk(system)
            .map(|disk| Target::Disk(disk.device.clone())),
    }
}

pub fn target(
    facts: &RepairFacts,
    picked: Option<&Target>,
    system: Option<&System>,
) -> Option<TargetChoice> {
    let choices = targets(facts);
    let listed = |wanted: &Target| {
        choices
            .iter()
            .find(|choice| choice.target == *wanted)
            .cloned()
    };

    picked
        .and_then(listed)
        .or_else(|| usual_target(facts, system?).as_ref().and_then(listed))
}

/// `None` for a system that has only one layout.
pub fn layout(system: &System, picked: Option<RootLayout>) -> Option<RootLayout> {
    system
        .asks_layout()
        .then(|| picked.unwrap_or_else(|| system.usual_layout()))
}
