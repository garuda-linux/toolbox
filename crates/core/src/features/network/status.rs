use toolbox_plan::{Plan, RootAction, TargetList, UnitName, UnitOperation};
pub use toolbox_state::network::{
    Connectivity, DefaultRoute, Radio, RadioAccess, RadioKind, RadioSummary, RadioTarget,
    public_address, rfkill_arguments,
};
use toolbox_state::services::{ActiveState, FileState, UnitInfo};

/// The units of the services tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkUnit {
    NetworkManager,
    Connman,
    Bluetooth,
    /// The location service desktops ask for the position. It starts on
    /// demand over D-Bus and has no enabled state of its own.
    Geoclue,
}

impl NetworkUnit {
    /// In the order the tab lists them.
    pub const ALL: [Self; 4] = [
        Self::NetworkManager,
        Self::Connman,
        Self::Bluetooth,
        Self::Geoclue,
    ];

    /// The name systemd lists the unit under.
    pub const fn unit(self) -> &'static str {
        match self {
            Self::NetworkManager => "NetworkManager.service",
            Self::Connman => "connman.service",
            Self::Bluetooth => "bluetooth.service",
            Self::Geoclue => "geoclue.service",
        }
    }
}

/// One service of the tab.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceRow {
    pub unit: NetworkUnit,
    pub active: ActiveState,
    pub file_state: FileState,
}

pub fn radio_summary(radios: &[Radio], kind: RadioKind) -> RadioSummary {
    let mut of_kind = radios.iter().filter(|radio| radio.kind == Some(kind));
    let Some(first) = of_kind.next() else {
        return RadioSummary::default();
    };

    of_kind.fold(
        RadioSummary {
            present: true,
            blocked: first.soft || first.hard,
            hard: first.hard,
        },
        |summary, radio| RadioSummary {
            present: true,
            blocked: summary.blocked && (radio.soft || radio.hard),
            hard: summary.hard || radio.hard,
        },
    )
}

/// On when the machine has a Wi-Fi or Bluetooth radio and every one of
/// them is blocked.
pub fn flight_mode(radios: &[Radio]) -> bool {
    let mut shown = radios
        .iter()
        .filter(|radio| radio.kind.is_some())
        .peekable();

    shown.peek().is_some() && shown.all(|radio| radio.soft || radio.hard)
}

/// The installed ones of [`SERVICES`] and [`GEOCLUE`] as systemd lists
/// them. A unit systemd does not list, or lists without a unit file, is
/// not installed and left out.
pub fn service_rows(units: &[UnitInfo]) -> Vec<ServiceRow> {
    NetworkUnit::ALL
        .into_iter()
        .filter_map(|network| {
            let unit = units.iter().find(|unit| {
                unit.name.as_str() == network.unit() && unit.file_state != FileState::Unknown
            })?;

            Some(ServiceRow {
                unit: network,
                active: unit.active,
                file_state: unit.file_state,
            })
        })
        .collect()
}

/// Starts a service at once in the Console. `None` for the location
/// service: the Services page starts that one.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_start(unit: NetworkUnit) -> Option<Plan> {
    if unit == NetworkUnit::Geoclue {
        return None;
    }

    let service = UnitName::new(unit.unit()).ok()?;

    Some(Plan::root(RootAction::ControlUnits {
        operation: UnitOperation::Start,
        units: TargetList::one(service),
    }))
}
