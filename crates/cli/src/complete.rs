//! The shell hooks call the binary with `COMPLETE=<shell>`, and [`run`] answers before anything else starts.
//! A [`Source`] must be quick, never need root or the helper, and may give no names.

#[cfg(test)]
mod tests;

use std::ffi::OsStr;
use std::panic::{self, catch_unwind};

use clap_complete::CompleteEnv;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use clap_complete::env::Shells;

const BIN: &str = "garuda-toolbox";

const VAR: &str = "COMPLETE";

/// An autoloaded zsh file runs as the function `_garuda-toolbox`. Its
/// first Tab would only register the completer, so this also answers it.
const ZSH_AUTOLOAD: &str = r#"
if [ "$funcstack[1]" = "_garuda-toolbox" ]; then
    _clap_dynamic_completer_garuda_toolbox "$@"
fi
"#;

const UNIT: Names = Names::Services { user: false };

const PLACES: &[Place] = &[
    (&[], "page", Names::Pages),
    (&["apps", "list"], "category", Names::AppCategories),
    (&["services", "start"], "unit", UNIT),
    (&["services", "stop"], "unit", UNIT),
    (&["services", "restart"], "unit", UNIT),
    (&["services", "reload"], "unit", UNIT),
    (&["services", "enable"], "unit", UNIT),
    (&["services", "disable"], "unit", UNIT),
    (&["services", "mask"], "unit", UNIT),
    (&["services", "unmask"], "unit", UNIT),
    (&["toggle", "on"], "id", Names::Switches),
    (&["toggle", "off"], "id", Names::Switches),
    (&["apps", "install"], "id", Names::Apps),
    (&["apps", "remove"], "id", Names::Apps),
    (&["kernels", "install"], "kernel", Names::Kernels),
    (&["kernels", "remove"], "kernel", Names::Kernels),
    (&["kernels", "headers"], "kernel", Names::Kernels),
    (&["kernels", "default"], "kernel", Names::Kernels),
    (&["dkms", "build"], "kernel", Names::Kernels),
    (&["maintenance", "run"], "task", Names::Tasks),
    (&["locales", "enable"], "locale", Names::Locales),
    (&["locales", "disable"], "locale", Names::Locales),
    (&["locales", "system"], "locale", Names::Locales),
    (&["users", "add"], "groups", Names::Groups),
    (&["users", "add"], "shell", Names::Shells),
    (&["users", "edit"], "user", Names::Users),
    (&["users", "edit"], "shell", Names::Shells),
    (&["users", "edit"], "add_groups", Names::Groups),
    (&["users", "edit"], "remove_groups", Names::Groups),
    (&["users", "password"], "user", Names::Users),
    (&["users", "lock"], "user", Names::Users),
    (&["users", "unlock"], "user", Names::Users),
    (&["users", "remove"], "user", Names::Users),
    (&["groups", "remove"], "group", Names::Groups),
    (&["boot", "set"], "key", Names::BootKeys),
    (&["boot", "set"], "value", Names::BootValues),
    (&["settings", "get"], "key", Names::SettingKeys),
    (&["settings", "set"], "key", Names::SettingKeys),
    (&["settings", "set"], "value", Names::SettingValues),
    (&["configs", "show"], "key", Names::ConfigFiles),
    (&["configs", "edit"], "key", Names::ConfigFiles),
    (
        &["network", "drivers", "blacklist"],
        "module",
        Names::Drivers,
    ),
    (&["network", "drivers", "allow"], "module", Names::Drivers),
    (&["packages", "install"], "packages", Names::OfferedPackages),
    (&["packages", "remove"], "package", Names::InstalledPackages),
    (&["packages", "show"], "package", Names::InstalledPackages),
    (
        &["locales", "packs", "install"],
        "package",
        Names::LanguagePacks { installed: false },
    ),
    (
        &["locales", "packs", "remove"],
        "package",
        Names::LanguagePacks { installed: true },
    ),
    (&["guide", "show"], "page", Names::GuidePages),
    (&["setup", "apply"], "ids", Names::SetupChoices),
    (
        &["boot", "param", "remove"],
        "parameter",
        Names::KernelParameters,
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Names {
    /// Units, your own with `user`.
    Services {
        user: bool,
    },
    Switches,
    Apps,
    Kernels,
    Locales,
    Users,
    Groups,
    ConfigFiles,
    Tasks,
    /// The boot menu's entries, for `boot set default`.
    BootEntries,
    Drivers,
    /// The window's pages, for `--page`.
    Pages,
    AppCategories,
    /// The login shells `/etc/shells` allows.
    Shells,
    OnOff,
    LogLevels,
    BootKeys,
    BootValues,
    SettingKeys,
    SettingValues,
    /// `system` and the app's own colour schemes of dark or light mode.
    ColourSchemes {
        dark: bool,
    },
    /// The packages the sync databases offer, as last synced.
    OfferedPackages,
    InstalledPackages,
    /// The AUR's packages, from the local copy of its package list.
    AurPackages,
    /// The language packs of the enabled locales, the installed ones with
    /// `installed`.
    LanguagePacks {
        installed: bool,
    },
    /// The guide built into the app, by file name without `.md`.
    GuidePages,
    SetupChoices,
    /// The kernel parameters the boot menu sets now.
    KernelParameters,
}

/// The names of a kind, as the app finds them now.
pub type Source = fn(Names) -> Vec<String>;

type Place = (&'static [&'static str], &'static str, Names);

/// Call it before anything prints, logs or checks root.
pub fn run(source: Source) {
    CompleteEnv::with_factory(move || command(source)).complete();
}

/// The script calls the binary on the path for each Tab, so the names stay current.
pub fn registration(shell: clap_complete::Shell, out: &mut dyn std::io::Write) {
    let name = shell.to_string();
    let shells = Shells::builtins();
    let Some(completer) = shells.completer(&name) else {
        return;
    };

    completer.write_registration(VAR, BIN, BIN, BIN, out).ok();
    if completer.name() == "zsh" {
        out.write_all(ZSH_AUTOLOAD.as_bytes()).ok();
    }
}

pub fn command(source: Source) -> clap::Command {
    PLACES
        .iter()
        .fold(crate::command(), |command, &(path, arg, names)| {
            let completer =
                ArgValueCompleter::new(move |current: &OsStr| candidates(source, names, current));

            attach(command, path, arg, completer)
        })
}

/// A place that does not exist leaves the command unchanged.
fn attach(
    command: clap::Command,
    path: &[&str],
    arg: &str,
    completer: ArgValueCompleter,
) -> clap::Command {
    match path.split_first() {
        None if command.get_arguments().any(|known| known.get_id() == arg) => {
            command.mut_arg(arg, |known| known.add(completer))
        }
        Some((first, rest)) if command.find_subcommand(first).is_some() => {
            command.mut_subcommand(first, |sub| attach(sub, rest, arg, completer))
        }
        None | Some(_) => command,
    }
}

/// A panicking source gives none, quietly, since the shell shows whatever a completer prints.
fn candidates(source: Source, names: Names, current: &OsStr) -> Vec<CompletionCandidate> {
    let words: Vec<String> = typed()
        .into_iter()
        .filter_map(|word| word.into_string().ok())
        .collect();

    let (Some(current), Some(names)) = (current.to_str(), wanted(names, &words)) else {
        return Vec::new();
    };

    panic::set_hook(Box::new(|_| {}));
    let found = catch_unwind(|| source(names)).unwrap_or_default();

    matching(found, current)
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

/// The words of the command line being completed, which the hooks pass
/// after `--`.
fn typed() -> Vec<std::ffi::OsString> {
    std::env::args_os()
        .skip_while(|word| word != "--")
        .skip(1)
        .collect()
}

fn wanted(names: Names, words: &[String]) -> Option<Names> {
    match names {
        Names::Services { user: _ } => Some(Names::Services {
            user: words.iter().any(|word| word == "--user"),
        }),
        Names::BootValues => match after_set(words) {
            Some("default") => Some(Names::BootEntries),
            Some(
                "remember" | "submenus" | "os-prober" | "quiet" | "splash" | "mitigations-off"
                | "sysrq",
            ) => Some(Names::OnOff),
            Some(_) | None => None,
        },
        Names::SettingValues => match after_set(words) {
            Some("start-at-login" | "user-services" | "copy-reports") => Some(Names::OnOff),
            Some("log-level") => Some(Names::LogLevels),
            Some("dark-scheme") => Some(Names::ColourSchemes { dark: true }),
            Some("light-scheme") => Some(Names::ColourSchemes { dark: false }),
            Some(_) | None => None,
        },
        Names::BootEntries => (after_set(words) == Some("default")).then_some(names),
        Names::OfferedPackages | Names::InstalledPackages => {
            if words.iter().any(|word| word == "--aur") {
                Some(Names::AurPackages)
            } else {
                Some(names)
            }
        }
        Names::AurPackages
        | Names::Switches
        | Names::Apps
        | Names::Kernels
        | Names::Locales
        | Names::Users
        | Names::Groups
        | Names::ConfigFiles
        | Names::Tasks
        | Names::Drivers
        | Names::Pages
        | Names::AppCategories
        | Names::Shells
        | Names::OnOff
        | Names::LogLevels
        | Names::BootKeys
        | Names::SettingKeys
        | Names::LanguagePacks { installed: _ }
        | Names::ColourSchemes { dark: _ }
        | Names::GuidePages
        | Names::SetupChoices
        | Names::KernelParameters => Some(names),
    }
}

fn after_set(words: &[String]) -> Option<&str> {
    words
        .iter()
        .skip_while(|word| *word != "set")
        .nth(1)
        .map(String::as_str)
}

fn matching(found: Vec<String>, current: &str) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();

    found
        .into_iter()
        .filter(|name| name.starts_with(current) && seen.insert(name.clone()))
        .collect()
}
