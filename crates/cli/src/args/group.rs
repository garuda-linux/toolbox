//! A group of commands that runs one of them when none is named:
//! `tweaks` lists, `queue` shows. The default's flags go on the group, so
//! `tweaks --json` is `tweaks list --json`.

use clap::{ArgMatches, Args, FromArgMatches, Subcommand};

use super::{
    BootCommand, GuideCommand, NetworkCommand, QueueCommand, Read, ReadList, SettingsCommand,
    TweaksCommand,
};

macro_rules! defaulted {
    ($group:ty, $args:ty, $variant:ident) => {
        impl Defaulted for $group {
            type Default = $args;

            fn by_default(args: $args) -> Self {
                Self::$variant(args)
            }
        }
    };
}

/// The command of the group, the default one when none was named.
#[derive(Debug)]
pub struct Group<C>(pub C);

/// Commands with one that runs when none is named.
pub trait Defaulted: Subcommand {
    type Default: Args;

    fn by_default(args: Self::Default) -> Self;
}

defaulted!(TweaksCommand, ReadList, List);
defaulted!(SettingsCommand, ReadList, List);
defaulted!(GuideCommand, ReadList, List);
defaulted!(NetworkCommand, Read, Show);
defaulted!(BootCommand, Read, Show);
defaulted!(QueueCommand, Read, Show);

impl<C: Defaulted> FromArgMatches for Group<C> {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let command = match matches.subcommand_name() {
            Some(_) => C::from_arg_matches(matches)?,
            None => C::by_default(C::Default::from_arg_matches(matches)?),
        };

        Ok(Self(command))
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(matches)?;
        Ok(())
    }
}

impl<C: Defaulted> Args for Group<C> {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        C::augment_subcommands(C::Default::augment_args(cmd)).subcommand_required(false)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        C::augment_subcommands_for_update(C::Default::augment_args_for_update(cmd))
            .subcommand_required(false)
    }
}
