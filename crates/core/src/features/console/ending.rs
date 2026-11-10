use toolbox_plan::protocol::Mode;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Ending {
    #[default]
    Applied,
    /// Some steps found nothing to do, such as a file already as wanted.
    Partly {
        skipped: usize,
    },
    Nothing,
    /// A dry run: no step ran, whatever the steps report.
    Tested,
}

impl Ending {
    pub const fn of(mode: Mode, done: usize, skipped: usize) -> Self {
        let dry_run = mode.is_dry_run();
        match (dry_run, done, skipped) {
            (true, _, _) => Self::Tested,
            (false, _, 0) => Self::Applied,
            (false, 0, _) => Self::Nothing,
            (false, _, skipped) => Self::Partly { skipped },
        }
    }
}
