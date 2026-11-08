use toolbox_plan::{GrubState, PlymouthTheme};

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
