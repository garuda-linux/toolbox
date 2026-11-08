#[cfg(test)]
mod tests;

use toolbox_plan::sdboot::LoaderSettings;
use toolbox_plan::{GrubState, KernelParameters, PlymouthTheme};

/// efivarfs puts the variable's attributes before its value.
const ATTRIBUTE_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootLoader {
    Grub,
    SystemdBoot,
    Unknown,
}

impl BootLoader {
    /// `LoaderInfo` names the loader that started this boot, so it wins over
    /// files another loader left behind. GRUB sets no such variable.
    pub fn detect(
        loader_info: Option<&[u8]>,
        grub_defaults: bool,
        install_conf: Option<&str>,
    ) -> Self {
        if loader_info
            .and_then(loader_name)
            .is_some_and(|name| name.starts_with("systemd-boot "))
        {
            return Self::SystemdBoot;
        }

        let uki =
            install_conf.is_some_and(|conf| conf.lines().any(|line| line.trim() == "layout=uki"));

        if uki {
            Self::SystemdBoot
        } else if grub_defaults {
            Self::Grub
        } else {
            Self::Unknown
        }
    }
}

/// The command line kernel-install builds into every UKI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelCmdline {
    /// `None` when a word cannot be written back word for word, such as a
    /// quoted value: the page leaves the line alone then.
    pub parameters: Option<KernelParameters>,
    pub line: String,
    /// Read from `/proc/cmdline`, because `/etc/kernel/cmdline` is missing
    /// and kernel-install takes the running one then.
    pub running: bool,
}

impl KernelCmdline {
    pub fn read(text: &str, running: bool) -> Self {
        Self {
            parameters: KernelParameters::parse(text),
            line: text.trim().to_owned(),
            running,
        }
    }
}

/// One part for both, because one UKI rebuild carries a new command line
/// and a new Plymouth theme alike. Only on systemd-boot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SdBootKernel {
    pub cmdline: KernelCmdline,
    /// `None` without Plymouth or without a theme named.
    pub theme: Option<PlymouthTheme>,
}

/// Only root may read the ESP, so this joins the snapshot once the Boot
/// page's menu read brings it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CurrentLoader(pub LoaderSettings);

/// What `/etc/default/grub` says. Missing from the snapshot on a system
/// without the file: GRUB is not its boot loader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentGrub(pub GrubState);

/// The installed Plymouth themes and the one in use. Missing from the
/// snapshot without Plymouth.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlymouthState {
    pub themes: Vec<PlymouthTheme>,
    /// `None` when no config file names one.
    pub current: Option<PlymouthTheme>,
}

/// Whether the system runs in a virtual machine, where the boot splash
/// works in a limited way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VirtualMachine(pub bool);

fn loader_name(variable: &[u8]) -> Option<String> {
    let units: Vec<u16> = variable
        .get(ATTRIBUTE_BYTES..)?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .take_while(|unit| *unit != 0)
        .collect();

    String::from_utf16(&units).ok()
}
