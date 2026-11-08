//! The arguments of `guide`: the user guide's pages, read in the terminal.

use clap::{Args, Subcommand};
use toolbox_plan::PageName;

use super::{Read, ReadList, name};

#[derive(Debug, Subcommand)]
pub enum GuideCommand {
    /// List the pages of the user guide, in the order of its parts.
    #[command(after_help = "Example:\n  garuda-toolbox guide list --search kernel")]
    List(ReadList),

    /// Print a page of the user guide.
    #[command(after_help = "Example:\n  garuda-toolbox guide show kernels")]
    Show(GuideShow),
}

#[derive(Debug, Args)]
pub struct GuideShow {
    /// The page, as `guide list` names it.
    #[arg(value_name = "PAGE", value_parser = name)]
    pub page: PageName,

    #[command(flatten)]
    pub read: Read,
}
