use std::borrow::Cow;

use egui::{Context, Label, Ui};
use toolbox_core::features::toggles::shown::Tally;
use toolbox_gui_kit::fonts::{self, BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ListItem, RowHeights, RowList, RowState, Select, Tone, dimmed, highlighted, row_height,
    section_heading, split_row, status_dot, switch, text_block_height,
};
use toolbox_gui_kit::{UiExt as _, icons, tokens};
use toolbox_plan::{GarudaUpdateFlag, ShellPath};

use super::view::{Choices, Flip, Pick};
use crate::command::TweakCommand;
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::SWITCH_WIDTH;
use crate::pages::toggles::tally_text;

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
        estimate: row_height(ui, text_block_height(ui, 1)),
    }
}

/// What is on of the "Shell and network" rows `query` matches. Only ad
/// blocking is a switch, the pickers are never on.
pub fn shell_network_tally(ctx: &Context, choices: &Choices, query: &str) -> Tally {
    Choice::ALL
        .into_iter()
        .filter(|choice| matches(ctx, choice.text_id(), query))
        .map(|choice| Tally {
            on: usize::from(matches!(choice, Choice::Hblock) && choices.hblock.on),
            total: 1,
        })
        .sum()
}

pub fn shell_network(
    ui: &mut Ui,
    context: &PageContext<'_>,
    choices: &Choices,
    query: &str,
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
    let count = tally_text(&ctx, shell_network_tally(&ctx, choices, query));
    section_heading(ui, icons::NETWORK_WIRED, &title, &count);
    ui.add_space(ui.tokens().sizes.heading_gap);
    let _rows = tracing::trace_span!("rows").entered();
    RowList::new(super::SHELL_NETWORK, &title, heights(ui)).show(
        ui,
        &matching,
        |ui, tweak, state| {
            let choice = tweak.what;
            let row = (choice.text_id(), query);
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
                        (choice.text_id(), state, query),
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
    query: &str,
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

    let tally = Tally {
        on: matching.iter().filter(|(_, flip, _)| flip.on).count(),
        total: matching.len(),
    };

    let count = tally_text(&ctx, tally);

    let title = i18n::text(&ctx, "tweaks-garuda-update");
    section_heading(ui, icons::SYSTEM_SOFTWARE_UPDATE, &title, &count);
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
            flip_row(ui, context, (&tweak.id, state, query), flip, |on| {
                TweakCommand::Flag(flag, on)
            });
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
    (id, query): (&str, &str),
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

    let placeholder = i18n::text(
        &ctx,
        if pick.known {
            "tweaks-other"
        } else {
            "tweaks-unavailable"
        },
    );

    dimmed(ui, context.busy, |ui| {
        split_row(
            ui,
            NARROW_CONTROL,
            |ui| text(ui, (id, query), pick.queued.then_some("tweaks-queued")),
            |ui| {
                let picked = ui
                    .add_enabled_ui(pick.known && !context.busy, |ui| {
                        Select::new(&title, &names)
                            .selected(pick.shown)
                            .placeholder(&placeholder)
                            .detail(detail)
                            .width(NARROW_CONTROL)
                            .show(ui)
                    })
                    .inner;

                if let Some(entry) = picked.and_then(|index| entries.get(index)) {
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
    (id, state, query): (&str, &RowState, &str),
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
        split_row(
            ui,
            SWITCH_WIDTH,
            |ui| text(ui, (id, query), tag),
            |ui| {
                let enabled = flip.known && !context.busy;
                let flipped = switch(ui, flip.on, &title, enabled).clicked();
                if flipped || (state.toggle && enabled) {
                    context.commands.send(command(!flip.on));
                }
            },
        );
    });
}

fn text(ui: &mut Ui, (id, query): (&str, &str), queued: Option<&str>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    toolbox_gui_kit::widgets::title_line(ui, true, |ui| {
        let title = i18n::text(&ctx, id);
        let strong = fonts::font(ui.style(), Weight::SemiBold, BODY);
        ui.label(highlighted(ui, &title, query, strong, tokens.text));
        if let Some(queued) = queued {
            status_dot(ui, &i18n::text(&ctx, queued), Tone::Accent);
        }
    });

    let description = i18n::text(&ctx, &format!("{id}-description"));
    let small = fonts::font(ui.style(), Weight::Regular, SMALL);
    let description = highlighted(ui, &description, query, small, tokens.secondary);
    ui.add(Label::new(description).wrap());
}

/// Whether `query` (trimmed, lower case, empty for all) is in the title
/// or description of `id`, as the user reads them.
pub(super) fn matches(ctx: &Context, id: &str, query: &str) -> bool {
    query.is_empty()
        || [
            i18n::text(ctx, id),
            i18n::text(ctx, &format!("{id}-description")),
        ]
        .iter()
        .any(|text| text.to_lowercase().contains(query))
}
