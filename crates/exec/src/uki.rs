#[cfg(test)]
mod tests;

use std::path::Path;

use toolbox_plan::procedure::{Command, InitramfsRoot, in_root, uki_rebuilds};

pub fn rebuild(system: &Path, root: InitramfsRoot) -> Option<Command> {
    let below = system.join(root.folder());

    uki_rebuilds()
        .into_iter()
        .find(|command| command.program().below(&below).is_file())
        .map(|command| in_root(command, root))
}
