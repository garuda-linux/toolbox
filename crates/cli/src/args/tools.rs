//! The commands that talk about the command line itself: a shell's
//! completion hook and the long help of one command.

use clap::Args;

#[derive(Debug, Args)]
pub struct Completions {
    /// The shell to print the completion hook for.
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}

#[derive(Debug, Args)]
pub struct Help {
    /// A command to explain, such as `users add`. Without one, the whole.
    #[arg(value_name = "COMMAND", num_args = 0..)]
    pub command: Vec<String>,
}
