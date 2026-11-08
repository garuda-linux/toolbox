//! GRUB as the Boot options page sees it: the variables of
//! `/etc/default/grub`, the settings the page may change there, and the
//! menu of `grub.cfg`. Pure text work: the probe reads with it, the helper
//! writes with it, so both agree on what a variable's value is.

mod defaults;
mod menu;
mod parameters;
mod setting;

pub use menu::{GrubMenu, MenuEntry};
pub use parameters::{InvalidKernelParameters, KernelParameters};
pub use setting::{
    DefaultEntry, GrubKey, GrubSetting, GrubSettings, GrubState, GrubTimeout, InvalidGrubSettings,
};

/// The file the page edits, relative to `/`. A system without it does
/// not boot with GRUB, and the helper never creates it.
pub const GRUB_DEFAULTS: &str = "etc/default/grub";

pub const GRUB_DEFAULTS_BACKUP: &str = "grub.bak";

/// The generated menu, relative to `/`. Usually readable by root alone.
pub const GRUB_CONFIG: &str = "boot/grub/grub.cfg";
