#[cfg(test)]
mod tests;

use std::path::Path;

use toolbox_plan::procedure::{Command, uki_rebuilds};

pub fn rebuild(system: &Path) -> Option<Command> {
    uki_rebuilds()
        .into_iter()
        .find(|command| command.program().below(system).is_file())
}
