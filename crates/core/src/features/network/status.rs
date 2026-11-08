use toolbox_plan::{Plan, RootAction, TargetList, UnitName, UnitOperation};
pub use toolbox_state::network::{
    Connectivity, DefaultRoute, Radio, RadioAccess, RadioKind, RadioSummary, RadioTarget,
    public_address, rfkill_arguments,
};
use toolbox_state::services::{ActiveState, FileState, UnitInfo};

pub const SERVICES: [&str; 3] = [
    "NetworkManager.service",
    "connman.service",
    "bluetooth.service",
];

/// The location service desktops ask for the position. It starts on
/// demand over D-Bus and has no enabled state of its own.
pub const GEOCLUE: &str = "geoclue.service";

/// One service of the tab.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceRow {
    pub unit: &'static str,
    /// systemd knows a unit file of that name.
    pub installed: bool,
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

/// [`SERVICES`] and [`GEOCLUE`] as systemd lists them. A unit systemd
/// does not list is not installed.
pub fn service_rows(units: &[UnitInfo]) -> Vec<ServiceRow> {
    SERVICES
        .into_iter()
        .chain([GEOCLUE])
        .map(|name| {
            let unit = units.iter().find(|unit| unit.name.as_str() == name);

            ServiceRow {
                unit: name,
                installed: unit.is_some_and(|unit| unit.file_state != FileState::Unknown),
                active: unit.map_or(ActiveState::Inactive, |unit| unit.active),
                file_state: unit.map_or(FileState::Unknown, |unit| unit.file_state),
            }
        })
        .collect()
}

/// Starts one of [`SERVICES`] at once in the Console. `None` for any
/// other unit: the Services page starts those.
#[must_use = "a plan does nothing until it is run"]
pub fn plan_start(unit: &str) -> Option<Plan> {
    let name = SERVICES.into_iter().find(|service| *service == unit)?;
    let service = UnitName::new(name).ok()?;

    Some(Plan::root(RootAction::ControlUnits {
        operation: UnitOperation::Start,
        units: TargetList::one(service),
    }))
}
