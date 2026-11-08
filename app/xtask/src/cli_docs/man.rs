//! The man pages: one per command down to the last level, each with the
//! version in its title, the value each option takes in its synopsis and
//! the pages it belongs to under SEE ALSO, and its other names under
//! ALIASES. The main page also lists the environment the program reads.

use std::fmt::Write as _;

use super::BIN;

/// The variables the program reads, each with what it does. The texts
/// are roff already. The developer switches `GARUDA_TOOLBOX_HELPER` and
/// `GARUDA_TOOLBOX_CATALOG_DIR` are left out, since a user never sets them.
const ENVIRONMENT: &[(&[&str], &str)] = &[
    (
        &["GARUDA_TOOLBOX_PAGER"],
        "The pager for lists and shows on a terminal, ahead of \\fBPAGER\\fR. \
         It is split into words without a shell. Empty or \\fBcat\\fR prints directly.",
    ),
    (
        &["PAGER"],
        "The pager when \\fBGARUDA_TOOLBOX_PAGER\\fR is unset. Without either, \\fBless \\-FRX\\fR.",
    ),
    (
        &["LESS"],
        "Given to the pager as \\fBFRX\\fR when unset, so less keeps the colours \
         and quits at once when the text fits one screen.",
    ),
    (
        &["LV"],
        "Given to the pager as \\fB\\-c\\fR when unset, for lv.",
    ),
    (
        &["NO_COLOR"],
        "Any value but an empty one leaves colour out, unless \\fB\\-\\-color always\\fR, \
         \\fBCLICOLOR_FORCE\\fR or \\fBFORCE_COLOR\\fR asks for it.",
    ),
    (
        &["CLICOLOR"],
        "\\fB0\\fR leaves colour out, as \\fBNO_COLOR\\fR does.",
    ),
    (
        &["CLICOLOR_FORCE", "FORCE_COLOR"],
        "A value other than \\fB0\\fR colours a pipe too.",
    ),
    (&["TERM"], "\\fBdumb\\fR leaves colour and the spinner out."),
    (
        &["COLUMNS"],
        "The terminal's width, which long cells are cut to. \\fB\\-\\-width\\fR comes first.",
    ),
    (
        &["VISUAL", "EDITOR"],
        "The editor of \\fBconfigs edit\\fR, when it is one of micro, nano, vim, nvim, \
         vi, hx, emacs, ne, joe or mcedit. Else the first installed of micro, nano and vi.",
    ),
    (
        &["XDG_CACHE_HOME"],
        "Where completion keeps the names of the system for ten seconds, \
         \\fI~/.cache\\fR without it.",
    ),
    (
        &["COMPLETE"],
        "Set by the completion hooks. The program then answers the completion \
         request of that shell and exits.",
    ),
    (
        &["GARUDA_TOOLBOX_LOG"],
        "Replaces the log level of the settings, in the filter syntax of tracing, \
         such as \\fBdebug\\fR.",
    ),
];

const ALSO: &[&str] = &["less(1)", "pacman(8)", "paru(8)", "systemctl(1)"];

pub fn pages(mut top: clap::Command) -> Vec<(String, Vec<u8>)> {
    top.build();
    let version = top.get_version().unwrap_or_default().to_owned();
    let mut made = Vec::new();

    collect(&top, BIN, &[], &version, &mut made);
    made
}

/// `parents` are the pages above it, the nearest last.
fn collect(
    command: &clap::Command,
    name: &str,
    parents: &[String],
    version: &str,
    made: &mut Vec<(String, Vec<u8>)>,
) {
    let named = command.clone().display_name(name.to_owned());
    made.push((format!("{name}.1"), page(&named, parents, version)));

    let parents: Vec<String> = parents.iter().cloned().chain([name.to_owned()]).collect();
    for sub in command.get_subcommands().filter(|sub| !sub.is_hide_set()) {
        let child = format!("{name}-{}", sub.get_name());
        collect(sub, &child, &parents, version, made);
    }
}

fn page(command: &clap::Command, parents: &[String], version: &str) -> Vec<u8> {
    let mut rendered = Vec::new();

    // Rendering into memory cannot fail.
    clap_mangen::Man::new(command.clone())
        .source(format!("{BIN} {version}"))
        .render(&mut rendered)
        .ok();

    let mut text = with_values(&String::from_utf8_lossy(&rendered), command);
    text.push_str(&aliases(command));
    if parents.is_empty() {
        text.push_str(&environment());
    }

    text.push_str(&see_also(parents));
    text.into_bytes()
}

/// `text` with the value of each option in the synopsis, which
/// `clap_mangen` names only under OPTIONS.
fn with_values(text: &str, command: &clap::Command) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let Some(at) = lines.iter().position(|line| line == ".SH SYNOPSIS") else {
        return text.to_owned();
    };

    if let Some(synopsis) = lines.get_mut(at + 1) {
        for arg in command.get_arguments().filter(|arg| !arg.is_hide_set()) {
            let (Some(long), Some(value)) = (arg.get_long(), value_of(arg)) else {
                continue;
            };

            let flag = format!("\\fB\\-\\-{}\\fR", escape(long));
            for end in ["]", ">"] {
                let given = format!("{flag} \\fI{}\\fR{end}", escape(&value));
                *synopsis = synopsis.replace(&format!("{flag}{end}"), &given);
            }
        }
    }

    let mut joined = lines.join("\n");
    joined.push('\n');
    joined
}

fn value_of(arg: &clap::Arg) -> Option<String> {
    if !arg.get_num_args()?.takes_values() {
        return None;
    }

    Some(arg.get_value_names().map_or_else(
        || arg.get_id().as_str().to_uppercase(),
        |names| {
            names
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        },
    ))
}

/// The other names of `command` as whole command lines, which
/// `clap_mangen` leaves out. Empty without a visible alias.
fn aliases(command: &clap::Command) -> String {
    let line = command.get_bin_name().unwrap_or_default();
    let before = line
        .rsplit_once(' ')
        .map_or(String::new(), |(words, _)| format!("{words} "));

    let names: Vec<String> = command
        .get_visible_aliases()
        .map(|alias| format!("\\fB{}\\fR", escape(&format!("{before}{alias}"))))
        .collect();

    if names.is_empty() {
        return String::new();
    }

    format!(".SH ALIASES\n{}\n", names.join(", "))
}

fn environment() -> String {
    let mut text = String::from(".SH ENVIRONMENT\n");

    for (names, what) in ENVIRONMENT {
        let names: Vec<String> = names.iter().map(|name| format!("\\fB{name}\\fR")).collect();

        // Writing into a `String` cannot fail.
        write!(text, ".TP\n{}\n{}\n", names.join(", "), apostrophes(what)).ok();
    }

    text
}

/// The pages above this one, the nearest first. The main page names the
/// programs it works with instead.
fn see_also(parents: &[String]) -> String {
    let pages: Vec<String> = if parents.is_empty() {
        ALSO.iter().map(|page| escape(page)).collect()
    } else {
        parents
            .iter()
            .rev()
            .map(|page| format!("{}(1)", escape(page)))
            .collect()
    };

    format!(".SH \"SEE ALSO\"\n{}\n", pages.join(", "))
}

fn escape(text: &str) -> String {
    apostrophes(&text.replace('\\', "\\\\").replace('-', "\\-"))
}

/// The apostrophe as the string every page defines first, as
/// `clap_mangen` writes it.
fn apostrophes(text: &str) -> String {
    text.replace('\'', "\\*(Aq")
}
