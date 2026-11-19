//! One copy against the file in use, with a pick per change.

use egui::{Frame, Label, ScrollArea, Ui};
use toolbox_core::features::merges::{Comparison, Part, Pick, Standing, choice, queued, standing};
use toolbox_gui_kit::UiExt as _;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Tone, banner, mono_small, pill_button, queued_tag, rich, segmented,
};
use toolbox_plan::{ConfigText, MergeCopy, Resolution};
use toolbox_system::MergePair;

use super::{MergeDialog, take_file_id, take_id};
use crate::command::HomeCommand;
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;

/// Unchanged lines shown before and after a change, as `diff -u` does.
const CONTEXT: usize = 3;

/// The buttons' row and the note under it.
const FOOTER: f32 = 96.0;

/// The texts as read, which a queued choice marks.
struct Files<'a> {
    copy: &'a MergeCopy,
    current: Option<&'a str>,
    copied: &'a str,
    context: &'a PageContext<'a>,
}

impl Files<'_> {
    fn queue(&self, resolution: Resolution) {
        self.context.commands.send(HomeCommand::Merge {
            copy: self.copy.clone(),
            choice: Some(choice(resolution, self.current, self.copied)),
        });
    }
}

pub(super) fn show(
    ui: &mut Ui,
    dialog: &mut MergeDialog,
    context: &PageContext<'_>,
    pair: &MergePair,
) {
    heading(ui, context, &pair.copy);

    let (copied, current) = match (&pair.copied, &pair.current) {
        (Ok(copied), Ok(current)) => (copied, current.as_ref()),
        (Err(_), _) | (_, Err(_)) => {
            let text = i18n::text(ui.ctx(), "home-merges-unreadable");
            banner(ui, Tone::Error, &text, &[]);
            return;
        }
    };

    let files = Files {
        copy: &pair.copy,
        current: current.map(ConfigText::as_str),
        copied: copied.as_str(),
        context,
    };

    standing_of(ui, dialog, &files);
    ui.add_space(ui.tokens().sizes.card_gap);
    note(ui, "home-merges-applied");
}

fn heading(ui: &mut Ui, context: &PageContext<'_>, copy: &MergeCopy) {
    let ctx = ui.ctx().clone();
    let heading = i18n::text_with(
        &ctx,
        "home-merges-pair",
        &[
            ("copy", Arg::from(copy.as_str())),
            ("file", Arg::from(copy.original().as_str())),
        ],
    );

    ui.add(Label::new(rich(ui, &heading, Weight::Medium, BODY, ui.tokens().text)).wrap());

    if queued(context.changes, copy).is_some() {
        ui.horizontal(|ui| {
            queued_tag(ui, &i18n::text(&ctx, "home-merges-queued"), Tone::Accent);
            let back = i18n::text(&ctx, "home-merges-take-back");
            if pill_button(ui, &back, ButtonKind::Ghost).clicked() {
                context.commands.send(HomeCommand::Merge {
                    copy: copy.clone(),
                    choice: None,
                });
            }
        });
    }

    ui.add_space(ui.tokens().sizes.card_gap);
}

fn standing_of(ui: &mut Ui, dialog: &mut MergeDialog, files: &Files<'_>) {
    let copy = files.copy;

    match (standing(copy, files.current, files.copied), files.current) {
        (Standing::Differs { changes: _ }, Some(current)) => {
            let comparison = Comparison::new(current, files.copied);
            changes(ui, dialog, copy, &comparison);
            buttons(ui, dialog, files, &comparison);
        }
        (Standing::CurrentGone, _) => {
            note(ui, "home-merges-gone");
            ui.horizontal(|ui| {
                let take = i18n::text(ui.ctx(), take_file_id(copy.kind()));
                if pill_button(ui, &take, ButtonKind::Soft).clicked() {
                    files.queue(Resolution::TakeCopy);
                }

                remove_button(ui, files);
            });
        }
        (
            standing @ (Standing::Same | Standing::OlderSave | Standing::Differs { changes: _ }),
            _,
        ) => {
            let id = if standing == Standing::Same {
                "home-merges-same"
            } else {
                "home-merges-older"
            };

            note(ui, id);
            remove_button(ui, files);
        }
    }
}

fn note(ui: &mut Ui, id: &str) {
    let text = i18n::text(ui.ctx(), id);
    ui.add(
        Label::new(rich(
            ui,
            &text,
            Weight::Regular,
            SMALL,
            ui.tokens().secondary,
        ))
        .wrap(),
    );
}

fn remove_button(ui: &mut Ui, files: &Files<'_>) {
    if pill_button(
        ui,
        &i18n::text(ui.ctx(), "home-merges-remove"),
        ButtonKind::Soft,
    )
    .clicked()
    {
        files.queue(Resolution::KeepCurrent);
    }
}

fn changes(ui: &mut Ui, dialog: &mut MergeDialog, copy: &MergeCopy, comparison: &Comparison<'_>) {
    let ctx = ui.ctx().clone();
    let keep = i18n::text(&ctx, "home-merges-keep");
    let take = i18n::text(&ctx, take_id(copy.kind()));
    let parts = comparison.parts();
    let height = (ui.available_height() - FOOTER).max(FOOTER);
    let mut change = 0;

    ScrollArea::vertical()
        .id_salt(("toolbox_gui::home::merges::changes", copy.as_str()))
        .max_height(height)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            for (index, part) in parts.iter().enumerate() {
                match part {
                    Part::Same(lines_of) => {
                        same(ui, lines_of, index == 0, index + 1 == parts.len());
                    }
                    Part::Change { current, copy: new } => {
                        lines(ui, current, '-', Tone::Error);
                        lines(ui, new, '+', Tone::Success);

                        let name = i18n::text_with(
                            &ctx,
                            "home-merges-change",
                            &[("number", Arg::from(change + 1))],
                        );

                        let picked = dialog.picks(copy).get(change).copied().unwrap_or_default();
                        let options = [(Pick::Keep, keep.as_ref()), (Pick::Take, take.as_ref())];
                        let salt = ("toolbox_gui::home::merges::pick", copy.as_str(), change);

                        if let Some(pick) = segmented(ui, salt, &name, &options, picked).value {
                            dialog.pick(copy, change, pick);
                        }

                        ui.add_space(ui.tokens().sizes.card_gap);
                        change += 1;
                    }
                }
            }
        });
}

fn same(ui: &mut Ui, lines_of: &[&str], first: bool, last: bool) {
    let head = if first {
        0
    } else {
        CONTEXT.min(lines_of.len())
    };

    let tail = if last {
        0
    } else {
        CONTEXT.min(lines_of.len() - head)
    };

    let hidden = lines_of.len() - head - tail;

    lines(
        ui,
        lines_of.get(..head).unwrap_or_default(),
        ' ',
        Tone::Neutral,
    );

    if hidden > 0 {
        let text = i18n::text_with(
            ui.ctx(),
            "home-merges-unchanged",
            &[("count", Arg::from(hidden))],
        );

        ui.add(Label::new(rich(
            ui,
            &text,
            Weight::Regular,
            SMALL,
            ui.tokens().secondary,
        )));
    }

    let rest = lines_of.get(lines_of.len() - tail..).unwrap_or_default();
    lines(ui, rest, ' ', Tone::Neutral);
}

fn lines(ui: &mut Ui, lines_of: &[&str], mark: char, tone: Tone) {
    if lines_of.is_empty() {
        return;
    }

    let tokens = ui.tokens();
    let fill = match tone {
        Tone::Error => tokens.banner_wash(tokens.error),
        Tone::Success => tokens.banner_wash(tokens.success),
        Tone::Neutral | Tone::Accent | Tone::Warning => egui::Color32::TRANSPARENT,
    };

    Frame::new().fill(fill).show(ui, |ui| {
        ui.set_width(ui.available_width());
        for line in lines_of {
            let shown = format!("{mark} {}", line.strip_suffix('\n').unwrap_or(line));
            ui.add(Label::new(mono_small(ui, &shown, tokens.text)).extend());
        }
    });
}

fn buttons(ui: &mut Ui, dialog: &mut MergeDialog, files: &Files<'_>, comparison: &Comparison<'_>) {
    let ctx = ui.ctx().clone();
    let label = |id: &str| i18n::text(&ctx, id);
    let copy = files.copy;
    let merged = comparison.resolution(dialog.picks(copy));

    ui.horizontal_wrapped(|ui| {
        if pill_button(ui, &label("home-merges-keep-file"), ButtonKind::Soft).clicked() {
            files.queue(Resolution::KeepCurrent);
        }

        if pill_button(ui, &label(take_file_id(copy.kind())), ButtonKind::Soft).clicked() {
            files.queue(Resolution::TakeCopy);
        }

        let picked = !matches!(merged, Ok(Resolution::KeepCurrent) | Err(_));
        let pressed = ui
            .enabled_ui(picked, |ui| {
                pill_button(ui, &label("home-merges-queue-picks"), ButtonKind::Primary)
            })
            .inner;

        if pressed.clicked()
            && let Ok(resolution) = merged.clone()
        {
            files.queue(resolution);
        }

        if pill_button(ui, &label("home-merges-edit"), ButtonKind::Ghost).clicked() {
            let start = match merged {
                Ok(Resolution::Merged { text }) => text.as_str().to_owned(),
                Ok(Resolution::KeepCurrent | Resolution::TakeCopy) | Err(_) => {
                    files.current.unwrap_or_default().to_owned()
                }
            };

            dialog.editing = Some((copy.clone(), start));
        }
    });
}
