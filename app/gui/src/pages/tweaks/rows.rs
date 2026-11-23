mod glyphs;
mod text;

use std::borrow::Cow;

use egui::{Context, Ui};
use toolbox_core::features::search::Needle;
use toolbox_core::features::toggles::shown::Tally;
use toolbox_core::features::tweaks::view::{Choices, Flip, Pick, Read};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::icons::Icon;
use toolbox_gui_kit::widgets::{
    ListItem, RowHeights, RowList, RowState, Select, dimmed, rich, row_height, section_heading,
    split_row, split_row_in, switch, text_block_height,
};
use toolbox_gui_kit::{UiExt as _, tokens};
use toolbox_plan::{GarudaUpdateFlag, ShellPath};

use self::glyphs::flag_glyph;
use self::text::lead;
pub(super) use self::text::matches;
use crate::command::TweakCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::{SWITCH_WIDTH, picture_height};
use crate::pages::toggles::shown_count;

const NARROW_CONTROL: f32 = tokens::Sizes::DEFAULT.setting_control_narrow;

struct Tweak<T> {
    id: Cow<'static, str>,
    title: Cow<'static, str>,
    what: T,
}

impl<T> Tweak<T> {
    fn new(ctx: &Context, id: Cow<'static, str>, what: T) -> Self {
        Self {
            title: i18n::text(ctx, &id),
            id,
            what,
        }
    }
}

impl<T> ListItem for Tweak<T> {
    fn key(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.title
    }
}

struct Entry {
    name: String,
    detail: Option<String>,
    command: TweakCommand,
}

#[derive(Clone, Copy, Debug)]
enum Choice {
    Shell,
    Dns,
    Wifi,
    Hblock,
}

impl Choice {
    const ALL: [Self; 4] = [Self::Shell, Self::Dns, Self::Wifi, Self::Hblock];

    /// The text id of the title. The description adds `-description`.
    const fn text_id(self) -> &'static str {
        match self {
            Self::Shell => "tweaks-shell",
            Self::Dns => "tweaks-dns",
            Self::Wifi => "tweaks-wifi",
            Self::Hblock => "tweaks-hblock",
        }
    }
}

/// The rows grow with a description that wraps, from the title and one
/// line of it.
fn heights(ui: &Ui) -> RowHeights {
    RowHeights::Measured {
        estimate: row_height(ui, text_block_height(ui, 1).max(picture_height(ui, true))),
    }
}

/// What is on of the "Shell and network" switches `query` matches, and
/// whether their state was read. Only ad blocking is one, a picker is
/// never on and so counts for nothing.
pub fn shell_network_tally(ctx: &Context, choices: &Choices, query: &Needle) -> (Tally, bool) {
    let shown = matches(ctx, Choice::Hblock.text_id(), query);
    let tally = Tally {
        on: usize::from(shown && choices.hblock.on),
        total: usize::from(shown),
    };

    (tally, choices.hblock.known)
}

/// What is on of the Garuda Update flags `query` matches, and whether
/// their state was read.
pub fn garuda_update_tally(
    ctx: &Context,
    flags: &[(GarudaUpdateFlag, Flip)],
    query: &Needle,
) -> (Tally, bool) {
    let matching = flags
        .iter()
        .filter(|(flag, _)| matches(ctx, &format!("tweaks-flag-{}", flag.as_str()), query));

    let tally = matching
        .clone()
        .map(|(_, flip)| Tally {
            on: usize::from(flip.on),
            total: 1,
        })
        .sum();

    (tally, matching.into_iter().all(|(_, flip)| flip.known))
}

/// The pickers whose probe failed, for the banner of what could not be
/// read. A queued wish shows what will be, as a catalog row's does.
pub fn unread(choices: &Choices) -> usize {
    [
        (choices.shell.read, choices.shell.queued),
        (choices.dns.read, choices.dns.queued),
        (choices.wifi.read, choices.wifi.queued),
    ]
    .into_iter()
    .filter(|&(read, queued)| read == Read::Failed && !queued)
    .count()
}

pub fn shell_network(
    ui: &mut Ui,
    context: &PageContext<'_>,
    choices: &Choices,
    query: &Needle,
) -> bool {
    let ctx = ui.ctx().clone();
    let matching: Vec<Choice> = Choice::ALL
        .into_iter()
        .filter(|choice| matches(&ctx, choice.text_id(), query))
        .collect();

    if matching.is_empty() {
        return false;
    }

    let matching: Vec<Tweak<Choice>> = matching
        .into_iter()
        .map(|choice| Tweak::new(&ctx, choice.text_id().into(), choice))
        .collect();

    let title = i18n::text(&ctx, "tweaks-shell-network");
    let (tally, read) = shell_network_tally(&ctx, choices, query);
    let count = shown_count(&ctx, tally, read).unwrap_or_default();
    section_heading(ui, &title, &count);
    ui.add_space(ui.tokens().sizes.heading_gap);
    let _rows = tracing::trace_span!("rows").entered();
    RowList::new(super::SHELL_NETWORK, &title, heights(ui)).show(
        ui,
        &matching,
        |ui, tweak, state| {
            let choice = tweak.what;
            let row = (choice.glyph(), choice.text_id(), query);
            match choice {
                Choice::Shell => {
                    pick_row(ui, context, row, &choices.shell, &shells(&choices.shell));
                }
                Choice::Dns => {
                    let entries = named(&choices.dns, |dns| {
                        let name = i18n::text(&ctx, &format!("tweaks-dns-{}", dns.as_str()));
                        (name.into_owned(), TweakCommand::Dns(*dns))
                    });

                    pick_row(ui, context, row, &choices.dns, &entries);
                }
                Choice::Wifi => {
                    let entries = named(&choices.wifi, |wifi| {
                        let name = i18n::text(&ctx, &format!("tweaks-wifi-{}", wifi.as_str()));
                        (name.into_owned(), TweakCommand::Wifi(*wifi))
                    });

                    pick_row(ui, context, row, &choices.wifi, &entries);
                }
                Choice::Hblock => {
                    flip_row(
                        ui,
                        context,
                        (choice.glyph(), choice.text_id(), state, query),
                        choices.hblock,
                        TweakCommand::Hblock,
                    );
                }
            }
        },
    );
    ui.add_space(ui.tokens().sizes.section_gap);

    true
}

/// Returns whether a row matched `query`. Without the configuration
/// there is no section.
pub fn garuda_update(
    ui: &mut Ui,
    context: &PageContext<'_>,
    choices: &Choices,
    query: &Needle,
) -> bool {
    let ctx = ui.ctx().clone();
    let Some(flags) = &choices.flags else {
        return false;
    };

    let matching: Vec<(GarudaUpdateFlag, Flip, String)> = flags
        .iter()
        .map(|(flag, flip)| (*flag, *flip, format!("tweaks-flag-{}", flag.as_str())))
        .filter(|(_, _, id)| matches(&ctx, id, query))
        .collect();

    if matching.is_empty() {
        return false;
    }

    let (tally, read) = garuda_update_tally(&ctx, flags, query);
    let count = shown_count(&ctx, tally, read).unwrap_or_default();

    let title = i18n::text(&ctx, "tweaks-garuda-update");
    section_heading(ui, &title, &count);
    ui.add_space(ui.tokens().sizes.heading_gap);
    let matching: Vec<Tweak<(GarudaUpdateFlag, Flip)>> = matching
        .into_iter()
        .map(|(flag, flip, id)| Tweak::new(&ctx, id.into(), (flag, flip)))
        .collect();

    let _rows = tracing::trace_span!("rows").entered();
    RowList::new(super::GARUDA_UPDATE, &title, heights(ui)).show(
        ui,
        &matching,
        |ui, tweak, state| {
            let (flag, flip) = tweak.what;
            let row = (flag_glyph(flag), tweak.id.as_ref(), state, query);
            flip_row(ui, context, row, flip, |on| TweakCommand::Flag(flag, on));
        },
    );
    ui.add_space(ui.tokens().sizes.section_gap);

    true
}

fn shells(pick: &Pick<ShellPath>) -> Vec<Entry> {
    ShellPath::names(&pick.options)
        .into_iter()
        .zip(&pick.options)
        .map(|(name, shell)| Entry {
            name,
            detail: Some(shell.as_str().to_owned()),
            command: TweakCommand::Shell(shell.clone()),
        })
        .collect()
}

fn named<T>(pick: &Pick<T>, entry: impl Fn(&T) -> (String, TweakCommand)) -> Vec<Entry> {
    pick.options
        .iter()
        .map(|option| {
            let (name, command) = entry(option);
            Entry {
                name,
                detail: None,
                command,
            }
        })
        .collect()
}

fn pick_row<T>(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (glyph, id, query): (Icon, &str, &Needle),
    pick: &Pick<T>,
    entries: &[Entry],
) {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, id);
    let names: Vec<&str> = entries.iter().map(|entry| entry.name.as_str()).collect();
    let detail = pick
        .shown
        .and_then(|index| entries.get(index))
        .and_then(|entry| entry.detail.as_deref());

    // Before the first probe the value is only not read yet, which a
    // toggle row's switch says with the same words.
    let placeholder = match pick.read {
        Read::Known => i18n::text(&ctx, "tweaks-other"),
        Read::Failed => i18n::text(&ctx, "tweaks-unread"),
        Read::Absent => i18n::text(&ctx, "tweaks-unavailable"),
        Read::NotYet => i18n::text(&ctx, "tweaks-not-read"),
    };

    let known = pick.read == Read::Known;
    let queued = pick.queued.then_some("tweaks-queued");
    dimmed(ui, context.busy, |ui| {
        split_row(
            ui,
            NARROW_CONTROL,
            |ui| lead(ui, (glyph, !known), (id, query), queued),
            |ui| {
                // Nothing to pick from is no control, so it reads as a
                // fact rather than a dropdown that never opens.
                if pick.read == Read::Absent {
                    let secondary = ui.tokens().secondary;
                    ui.label(rich(ui, &placeholder, Weight::Regular, BODY, secondary));
                    return;
                }

                let picked = ui
                    .enabled_ui(known && !context.busy, |ui| {
                        Select::new(&title, &names)
                            .selected(pick.shown)
                            .placeholder(&placeholder)
                            .detail(detail)
                            .width(NARROW_CONTROL)
                            .show(ui)
                    })
                    .inner;

                if let Some(entry) = picked.value.and_then(|index| entries.get(index)) {
                    context.commands.send(entry.command.clone());
                }
            },
        );
    });
}

/// Space flips the switch.
fn flip_row(
    ui: &mut Ui,
    context: &PageContext<'_>,
    (glyph, id, state, query): (Icon, &str, &RowState, &Needle),
    flip: Flip,
    command: impl Fn(bool) -> TweakCommand,
) {
    let title = i18n::text(ui.ctx(), id);
    let tag = flip.queued.then_some(if flip.on {
        "toggles-turns-on"
    } else {
        "toggles-turns-off"
    });

    dimmed(ui, context.busy, |ui| {
        split_row_in(
            ui,
            state.row,
            SWITCH_WIDTH,
            |ui| lead(ui, (glyph, !flip.known), (id, query), tag),
            |ui| {
                let enabled = flip.known && !context.busy;
                let flipped = ui
                    .enabled_ui(enabled, |ui| switch(ui, flip.on, &title).clicked())
                    .inner;

                if flipped || (state.toggle && enabled) {
                    context.commands.send(command(!flip.on));
                }
            },
        );
    });
}
