//! The kernel parameters the Boot options page has switches for, and the
//! free parameters beside them. The switches are views of the one command
//! line, so the free list hides what they own while the queue keeps it.

use std::sync::LazyLock;

use toolbox_plan::{KernelParameter, KernelParameters};

#[expect(clippy::expect_used, reason = "constant parameters, parsed by a test")]
static PARAMETERS: LazyLock<[KernelParameter; 4]> = LazyLock::new(|| {
    Flag::ALL.map(|flag| {
        KernelParameter::new(flag.parameter()).expect("a switch's parameter is a kernel parameter")
    })
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub enum Flag {
    Quiet,
    Splash,
    MitigationsOff,
    #[strum(serialize = "sysrq")]
    SysRq,
}

impl Flag {
    pub const ALL: [Self; 4] = [Self::Quiet, Self::Splash, Self::MitigationsOff, Self::SysRq];

    pub fn as_str(self) -> &'static str {
        self.into()
    }

    pub const fn parameter(self) -> &'static str {
        match self {
            Self::Quiet => "quiet",
            Self::Splash => "splash",
            Self::MitigationsOff => "mitigations=off",
            Self::SysRq => "sysrq_always_enabled=1",
        }
    }

    pub fn kernel_parameter(self) -> &'static KernelParameter {
        let [quiet, splash, mitigations_off, sysrq] = &*PARAMETERS;
        match self {
            Self::Quiet => quiet,
            Self::Splash => splash,
            Self::MitigationsOff => mitigations_off,
            Self::SysRq => sysrq,
        }
    }

    /// Every spelling the switch owns. A bare `sysrq` left by hand editing
    /// still reads as the `SysRq` switch being on.
    pub const fn spellings(self) -> &'static [&'static str] {
        match self {
            Self::Quiet => &["quiet"],
            Self::Splash => &["splash"],
            Self::MitigationsOff => &["mitigations=off"],
            Self::SysRq => &["sysrq_always_enabled=1", "sysrq"],
        }
    }

    /// Whether `parameter` is one of the switch's spellings. Only exact
    /// spellings count, so other `mitigations=` values stay free.
    pub fn matches(self, parameter: &str) -> bool {
        self.spellings().contains(&parameter)
    }

    pub fn is_set(self, list: &KernelParameters) -> bool {
        list.iter().any(|own| self.matches(own.as_str()))
    }

    pub fn stripped(self, list: &KernelParameters) -> KernelParameters {
        let mut stripped = list.clone();

        for spelling in self.spellings() {
            stripped = stripped.without(spelling);
        }

        stripped
    }

    /// The command line with the secondary spellings taken out, so
    /// switching on unifies on the canonical one without moving it.
    pub fn unified(self, list: &KernelParameters) -> KernelParameters {
        let mut unified = list.clone();

        for spelling in self.spellings() {
            if *spelling != self.parameter() {
                unified = unified.without(spelling);
            }
        }

        unified
    }
}

pub fn is_flag_parameter(parameter: &str) -> bool {
    Flag::ALL.iter().any(|flag| flag.matches(parameter))
}

/// The command line without what the switches own, in order. The queue
/// still holds the whole list, only the display hides the owned words.
pub fn free_parameters(list: &KernelParameters) -> Vec<&str> {
    list.iter()
        .map(KernelParameter::as_str)
        .filter(|word| !is_flag_parameter(word))
        .collect()
}
