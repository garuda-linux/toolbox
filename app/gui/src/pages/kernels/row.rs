use egui::{Context, Label, Response, Ui};
use toolbox_core::Toggle;
use toolbox_core::features::kernels::view::{Module, Row};
use toolbox_core::features::kernels::{ModuleState, Refusal, may_remove};
use toolbox_core::features::toggles::shown::Control;
use toolbox_gui_kit::fonts::{self, SMALL, Weight};
use toolbox_gui_kit::icons::Icon;
use toolbox_gui_kit::widgets::{
    ActionRow, ButtonKind, ListItem, RowState, Tone, actions_stacked, counted_disclosure,
    highlighted, named_pill_button, pill_button, pill_width, push_height, queued_tag, rich,
    row_height, text_block_height_with,
};
use toolbox_gui_kit::{UiExt as _, tokens};

use crate::command::{BootCommand, KernelEdit};
use crate::i18n;
use crate::i18n::Arg;
use crate::pages::PageContext;
use crate::pages::toggles::control::{self, Setting};

const LINES: u8 = 2;

/// A row of the list. `Row` and [`ListItem`] both come from other crates,
/// so the trait goes on this wrapper.
pub struct Item<'rows>(pub &'rows Row);

impl ListItem for Item<'_> {
    fn key(&self) -> &str {
        self.0.kernel.name.as_str()
    }

    fn name(&self) -> &str {
        self.0.kernel.name.as_str()
    }
}

/// A row's height before it was laid out, in the fonts in use. Kernels
/// with headers or modules grow below it.
pub fn estimate(ui: &Ui) -> f32 {
    row_height(ui, text_block_height_with(ui, LINES, push_height(ui)))
}

/// One kernel as an action row on its section's tile. Its words hold the
/// description, the versions and what is missing, its buttons the default
/// and the change.
pub fn show(ui: &mut Ui, context: &PageContext<'_>, row: &Row, glyph: Icon, state: &RowState) {
    let name = row.kernel.name.as_str();
    let details = row.kernel.is_installed() && row.queued != Some(Toggle::Off);
    let status = status(ui.ctx(), row);
    let default = row
        .default
        .then(|| i18n::text(ui.ctx(), "kernels-default").into_owned());

    let words = ActionRow::new(glyph, name).row(state.row);
    let words = match (&status, &default) {
        (Some((label, tone)), Some(default)) => words
            .status(label, *tone)
            .second_status(default, Tone::Success),
        (Some((label, tone)), None) => words.status(label, *tone),
        (None, Some(default)) => words.status(default, Tone::Success),
        (None, None) => words,
    };

    words.show_with_body(
        ui,
        |ui| {
            text(ui, row);
            if details {
                headers(ui, context, row);
                modules(ui, context, row);
            }
        },
        |ui| {
            if details && !row.default {
                default_link(ui, context, row);
            } else {
                default_slot(ui, state.row);
            }

            // Core refuses the running kernel and the last one left, so
            // their Remove rests instead of refusing on click.
            let offered = row.control();
            let removable = offered != Control::Remove
                || !matches!(
                    may_remove(&row.kernel.name, context.changes, context.snapshot),
                    Err(Refusal::Running | Refusal::LastKernel)
                );

            let shown = control::Shown {
                control: offered,
                on: row.wanted,
                title: name,
                enabled: removable && !context.busy,
                unknown: false,
                setting: Setting::Button,
            };

            if control::show(ui, shown, state) {
                context.commands.send(KernelEdit::Toggle {
                    kernel: row.kernel.name.clone(),
                    on: Toggle::from_on(!row.wanted),
                });
            }
        },
    );
}

/// The room of the default link, kept beside the words so the change buttons
/// line up. Under the words there is none to keep.
fn default_slot(ui: &mut Ui, row: egui::Id) {
    if !actions_stacked(ui, row) {
        let text = i18n::text(ui.ctx(), "kernels-set-default");
        ui.add_space(pill_width(ui, &text) + ui.tokens().sizes.control_gap);
    }
}

/// The state beside the name: a queued change, else that the kernel runs.
/// A default kernel's own status comes beside it.
fn status(ctx: &Context, row: &Row) -> Option<(String, Tone)> {
    match row.queued {
        Some(Toggle::On) => Some((
            i18n::text(ctx, "kernels-installs").into_owned(),
            Tone::Accent,
        )),
        Some(Toggle::Off) => Some((
            i18n::text(ctx, "kernels-removes").into_owned(),
            Tone::Accent,
        )),
        None => row.running.then(|| {
            (
                i18n::text(ctx, "kernels-running").into_owned(),
                Tone::Success,
            )
        }),
    }
}

fn text(ui: &mut Ui, row: &Row) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let kernel = &row.kernel;

    if !kernel.description.is_empty() {
        // The cut is the catalog rows' own, and the tooltip keeps the whole.
        let font = fonts::font(ui.style(), Weight::Regular, SMALL);
        let cut = control::cut(highlighted(
            ui,
            &kernel.description,
            "",
            font,
            tokens.secondary,
        ));

        ui.add(Label::new(cut).wrap())
            .on_hover_text(&kernel.description);
    }

    let mut version = if kernel.repo.is_empty() {
        i18n::text_with(
            &ctx,
            "kernels-version-local",
            &[("version", Arg::from(&kernel.version))],
        )
    } else {
        i18n::text_with(
            &ctx,
            "kernels-version",
            &[
                ("version", Arg::from(&kernel.version)),
                ("repo", Arg::from(&kernel.repo)),
            ],
        )
    };

    if let Some(installed) = kernel
        .installed
        .as_ref()
        .filter(|installed| **installed != kernel.version)
    {
        let note = i18n::text_with(
            &ctx,
            "kernels-version-installed",
            &[("installed", Arg::from(installed))],
        );

        version = i18n::text_with(
            &ctx,
            "kernels-version-note",
            &[("version", Arg::from(&version)), ("note", Arg::from(&note))],
        )
        .into_owned()
        .into();
    }

    ui.add(Label::new(rich(ui, &version, Weight::Regular, SMALL, tokens.secondary)).wrap());

    if row.running && kernel.is_installed() {
        let note = i18n::text(&ctx, "kernels-running-note");
        ui.add(Label::new(rich(ui, &note, Weight::Regular, SMALL, tokens.secondary)).wrap());
    }
}

/// Installed headers are the normal case, so only missing or queued ones
/// take a line.
fn headers(ui: &mut Ui, context: &PageContext<'_>, row: &Row) {
    if row.kernel.headers_installed {
        return;
    }

    let ctx = ui.ctx().clone();
    let queued = row.headers_queued == Some(Toggle::On);
    let (state, tone, action) = if queued {
        (
            "kernels-headers-queued",
            Tone::Accent,
            "kernels-unqueue-headers",
        )
    } else if row.lacks_headers {
        (
            "kernels-headers-missing",
            Tone::Warning,
            "kernels-queue-headers",
        )
    } else {
        return;
    };

    ui.add_space(ui.tokens().sizes.tight_gap);
    tags_line(ui, true, |ui| {
        queued_tag(ui, &i18n::text(&ctx, state), tone);
        if waiting_button(ui, context, &i18n::text(&ctx, action)).clicked() {
            context.commands.send(KernelEdit::Headers {
                kernel: row.kernel.name.clone(),
                on: Toggle::from_on(!queued),
            });
        }
    });
}

/// The modules fold under one line. A kernel with a build to do shows
/// them open until the user folds them, so what needs work is in sight.
fn modules(ui: &mut Ui, context: &PageContext<'_>, row: &Row) {
    if row.modules.is_empty() {
        return;
    }

    let ctx = ui.ctx().clone();
    let unbuilt = row.unbuilt() > 0;
    let id = ui.make_persistent_id(("kernel-modules", row.kernel.name.as_str()));
    let open = ctx
        .data(|data| data.get_temp::<bool>(id))
        .unwrap_or(unbuilt);

    ui.add_space(ui.tokens().sizes.tight_gap);
    tags_line(ui, unbuilt, |ui| {
        let title = i18n::text(&ctx, "kernels-modules");
        let count = row.modules.len().to_string();
        if counted_disclosure(ui, &title, &count, open).clicked() {
            ctx.data_mut(|data| data.insert_temp(id, !open));
        }

        if unbuilt {
            builds_button(ui, context, row);
        }
    });

    if !open {
        return;
    }

    ui.add_space(ui.tokens().sizes.tight_gap);
    tags_line(ui, false, |ui| {
        for module in &row.modules {
            let (text_id, tone) = module_text(module);
            let text = i18n::text_with(
                &ctx,
                text_id,
                &[
                    ("module", Arg::from(module.status.module.as_str())),
                    ("version", Arg::from(module.status.version.as_str())),
                ],
            );

            queued_tag(ui, &text, tone);
        }
    });
}

fn builds_button(ui: &mut Ui, context: &PageContext<'_>, row: &Row) {
    let ctx = ui.ctx().clone();
    let queued = row.builds_queued();
    let action = if queued {
        "kernels-unqueue-builds"
    } else {
        "kernels-queue-builds"
    };

    if waiting_button(ui, context, &i18n::text(&ctx, action)).clicked() {
        context.commands.send(KernelEdit::Builds {
            kernel: row.kernel.name.clone(),
            on: Toggle::from_on(!queued),
        });
    }
}

const fn module_text(module: &Module) -> (&'static str, Tone) {
    if module.queued {
        return ("kernels-module-queued", Tone::Accent);
    }

    match module.status.state {
        ModuleState::Installed => ("kernels-module-installed", Tone::Success),
        ModuleState::Built => ("kernels-module-built", Tone::Warning),
        ModuleState::Broken => ("kernels-module-broken", Tone::Error),
        ModuleState::Missing => ("kernels-module-missing", Tone::Warning),
    }
}

/// A wrapped line of tags, centred on a push button when `with_button`.
fn tags_line(ui: &mut Ui, with_button: bool, add: impl FnOnce(&mut Ui)) {
    ui.scope(|ui| {
        if with_button {
            ui.spacing_mut().interact_size.y = push_height(ui);
        }

        ui.horizontal_wrapped(add);
    });
}

fn waiting_button(ui: &mut Ui, context: &PageContext<'_>, text: &str) -> Response {
    // The enabled scope below is a child that never wraps, so the line
    // breaks here when the pill would not fit.
    if ui.available_size_before_wrap().x < pill_width(ui, text) {
        ui.end_row();
    }

    ui.enabled_ui(!context.busy, |ui| pill_button(ui, text, ButtonKind::Soft))
        .inner
}

/// Opens Boot options with this kernel's menu entry offered as the
/// default. Nothing changes until it is taken there.
fn default_link(ui: &mut Ui, context: &PageContext<'_>, row: &Row) {
    let ctx = ui.ctx().clone();
    let name = row.kernel.name.as_str();
    let text = i18n::text(&ctx, "kernels-set-default");
    let label = i18n::text_with(
        &ctx,
        "kernels-set-default-name",
        &[("kernel", Arg::from(name))],
    );

    let response = named_pill_button(ui, &text, ButtonKind::Soft, &label);

    if response.clicked() {
        context
            .commands
            .send(BootCommand::Propose(row.kernel.name.clone()));
    }
}
