use clap::{Args, Subcommand};
use toolbox_plan::MergeCopy;

use super::{Confirm, ReadList};

#[derive(Debug, Subcommand)]
pub enum MergesCommand {
    /// List the copies pacman left beside configuration files, as
    /// `pacdiff -o` finds them, and how each differs from its file.
    #[command(after_help = "Example:\n  garuda-toolbox merges list --json")]
    List(ReadList),

    /// Show what a copy changes against the file in use, as a unified diff.
    #[command(after_help = "Example:\n  garuda-toolbox merges show /etc/pacman.conf.pacnew")]
    Show(MergeShow),

    /// Keep the file in use and remove the copy, or put the copy in its
    /// place. A snapshot is taken first where snapper is set up.
    #[command(
        after_help = "Examples:\n  garuda-toolbox merges resolve /etc/pacman.conf.pacnew --take-new\n  \
                      garuda-toolbox merges resolve /etc/locale.gen.pacsave --keep"
    )]
    Resolve(MergeResolveArgs),
}

#[derive(Debug, Args)]
pub struct MergeShow {
    /// The copy, such as `/etc/pacman.conf.pacnew`.
    #[arg(value_parser = copy, value_name = "COPY")]
    pub copy: MergeCopy,
}

#[derive(Debug, Args)]
pub struct MergeResolveArgs {
    /// The copy, such as `/etc/pacman.conf.pacnew`.
    #[arg(value_parser = copy, value_name = "COPY")]
    pub copy: MergeCopy,

    #[command(flatten)]
    pub how: KeepOrTake,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Clone, Copy, Debug, Args)]
#[group(required = true, multiple = false)]
pub struct KeepOrTake {
    /// Keep the file in use and remove the copy.
    #[arg(long)]
    pub keep: bool,

    /// Put the copy in the file's place. The old file is kept in
    /// `/var/lib/garuda-toolbox/backups`.
    #[arg(long)]
    pub take_new: bool,
}

fn copy(text: &str) -> Result<MergeCopy, String> {
    MergeCopy::new(text).map_err(|error| error.to_string())
}
