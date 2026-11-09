//! Passwords never come as an argument: the terminal reads them without echo, or one line of standard input with `--password-stdin`.

use clap::{Args, Subcommand};
use toolbox_plan::{FullName, GroupName, HomeDirectory, ShellPath, UserName};

use super::{Confirm, ReadList};

#[derive(Debug, Subcommand)]
pub enum UsersCommand {
    /// List the people's accounts and root. `--all` adds the system ones.
    #[command(
        after_help = "Examples:\n  garuda-toolbox users list\n  garuda-toolbox users list --all --json"
    )]
    List(UsersList),

    /// Create an account and set its password, at once.
    #[command(
        after_help = "Examples:\n  garuda-toolbox users add ada --full-name \"Ada Lovelace\" \
                            --admin\n  \
                            printf '%s\\n' \"$PW\" | garuda-toolbox users add ci --password-stdin --yes"
    )]
    Add(AddUser),

    /// Change an account's full name, login shell or groups.
    #[command(
        after_help = "Examples:\n  garuda-toolbox users edit ada --shell /usr/bin/fish\n  \
                            garuda-toolbox users edit ada --add-group docker --remove-group video"
    )]
    Edit(EditUser),

    /// Set an account's password, at once.
    #[command(after_help = "Example:\n  garuda-toolbox users password ada")]
    Password(PasswordArgs),

    /// Lock an account's password, so it cannot log in with it.
    #[command(after_help = "Example:\n  garuda-toolbox users lock guest")]
    Lock(UserArgs),

    /// Unlock a locked account's password.
    #[command(after_help = "Example:\n  garuda-toolbox users unlock guest")]
    Unlock(UserArgs),

    /// Remove an account. Its home folder stays unless `--delete-home`.
    #[command(after_help = "Examples:\n  garuda-toolbox users remove guest\n  \
                            garuda-toolbox users remove guest --delete-home")]
    Remove(RemoveUser),
}

#[derive(Debug, Subcommand)]
pub enum GroupsCommand {
    /// List the groups and their members.
    #[command(after_help = "Example:\n  garuda-toolbox groups list --json")]
    List(ReadList),

    /// Create a group.
    #[command(after_help = "Example:\n  garuda-toolbox groups add media")]
    Add(GroupArgs),

    /// Remove a group.
    #[command(after_help = "Example:\n  garuda-toolbox groups remove media")]
    Remove(GroupArgs),
}

#[derive(Debug, Args)]
pub struct UserArgs {
    /// The account, as `users list` prints it.
    pub user: UserName,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct GroupArgs {
    /// The group, as `groups list` prints it.
    pub group: GroupName,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct AddUser {
    /// The new account's name: small letters, digits, hyphens and
    /// underscores, at most 32 characters.
    pub user: UserName,

    /// The person's full name.
    #[arg(long, value_name = "NAME")]
    pub full_name: Option<FullName>,

    /// The login shell, one of /etc/shells. Without it useradd's default.
    #[arg(long, value_name = "PATH")]
    pub shell: Option<ShellPath>,

    /// May install software and change the system (the group wheel).
    #[arg(long)]
    pub admin: bool,

    /// A further group, as often as needed.
    #[arg(long = "group", value_name = "GROUP")]
    pub groups: Vec<GroupName>,

    #[command(flatten)]
    pub password: PasswordSource,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct EditUser {
    /// The account, as `users list` prints it.
    pub user: UserName,

    /// The new full name. An empty one clears it.
    #[arg(long, value_name = "NAME")]
    pub full_name: Option<FullName>,

    /// The new login shell, one of /etc/shells.
    #[arg(long, value_name = "PATH")]
    pub shell: Option<ShellPath>,

    /// Join a group, as often as needed.
    #[arg(long = "add-group", value_name = "GROUP")]
    pub add_groups: Vec<GroupName>,

    /// Leave a group, as often as needed.
    #[arg(long = "remove-group", value_name = "GROUP")]
    pub remove_groups: Vec<GroupName>,

    /// A new home folder: its full path below /home, /srv, /var/lib or /opt.
    #[arg(long, value_name = "PATH")]
    pub home: Option<HomeDirectory>,

    /// Move the old home folder's files to the new one.
    #[arg(long, requires = "home")]
    pub move_files: bool,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct PasswordArgs {
    /// The account, as `users list` prints it.
    pub user: UserName,

    #[command(flatten)]
    pub password: PasswordSource,

    #[command(flatten)]
    pub confirm: Confirm,
}

#[derive(Debug, Args)]
pub struct RemoveUser {
    /// The account, as `users list` prints it.
    pub user: UserName,

    /// Delete the home folder too. It cannot be undone.
    #[arg(long)]
    pub delete_home: bool,

    #[command(flatten)]
    pub confirm: Confirm,
}

/// Where a password comes from. Never an argument: those land in the
/// shell's history and in the process list.
#[derive(Clone, Copy, Debug, Default, Args)]
pub struct PasswordSource {
    /// Read the password from the first line of standard input instead of
    /// asking on the terminal.
    #[arg(long)]
    pub password_stdin: bool,
}

#[derive(Debug, Args)]
pub struct UsersList {
    /// Every account, the system ones too.
    #[arg(long)]
    pub all: bool,

    #[command(flatten)]
    pub read: ReadList,
}
