//! The install of a ranked mirrorlist. The ranking takes minutes, so the
//! list may land while another plan runs. It waits for that plan's end
//! instead of being lost.

#[cfg(test)]
mod tests;

use toolbox_plan::Plan;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Install {
    #[default]
    None,
    /// Sent to the apply service, which has not started it yet.
    Sent(Plan),
    /// Refused, as another plan ran. Waits for that plan's end.
    Kept(Plan),
    /// The plan it waited for ended, so it goes out again.
    Due(Plan),
}

impl Install {
    /// A new list replaces one that waits, as it is the newer ranking.
    /// Returns the plan to send.
    pub fn landed(&mut self, plan: Plan) -> Plan {
        *self = Self::Sent(plan.clone());
        plan
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn started(self) -> Self {
        match self {
            Self::Sent(_) => Self::None,
            other @ (Self::None | Self::Kept(_) | Self::Due(_)) => other,
        }
    }

    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn refused(self) -> Self {
        match self {
            Self::Sent(plan) => Self::Kept(plan),
            other @ (Self::None | Self::Kept(_) | Self::Due(_)) => other,
        }
    }

    /// Some plan ended. A kept list goes out once it did, whichever plan
    /// it was: the one that ran is the one the list waited for.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn finished(self) -> Self {
        match self {
            Self::Kept(plan) => Self::Due(plan),
            other @ (Self::None | Self::Sent(_) | Self::Due(_)) => other,
        }
    }

    /// The plan to send now, if the list is due.
    pub fn due(&mut self) -> Option<Plan> {
        match std::mem::take(self) {
            Self::Due(plan) => Some(self.landed(plan)),
            other @ (Self::None | Self::Sent(_) | Self::Kept(_)) => {
                *self = other;
                None
            }
        }
    }
}
