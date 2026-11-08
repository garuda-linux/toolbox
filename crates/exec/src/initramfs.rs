//! Which program builds the initramfs again. Picked on the system itself,
//! since only it knows whether dracut or mkinitcpio builds them.

#[cfg(test)]
mod tests;

use std::path::Path;

use toolbox_plan::procedure::{Command, InitramfsRoot, initramfs_rebuilds};

pub fn rebuild(system: &Path, root: InitramfsRoot) -> Option<Command> {
    let below = system.join(root.folder());

    initramfs_rebuilds()
        .into_iter()
        .find(|command| command.program().below(&below).is_file())
        .map(|command| root.wrap(command))
}
