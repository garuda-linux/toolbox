use egui::{Label, Response, Ui};
use toolbox_core::Toggle;
use toolbox_core::features::kernels::ModuleState;
use toolbox_gui_kit::fonts::{BODY, SMALL, Weight};
use toolbox_gui_kit::theme::PILL_HEIGHT;
use toolbox_gui_kit::widgets::{
    ButtonKind, ListItem, RowState, Tone, named_pill_button, pill_button, pill_width, queued_tag,
    rich, row_height, split_row, status_dot, switch, text_block_height,
};
use toolbox_gui_kit::{UiExt as _, tokens};

use super::view::{Module, Row};
use crate::command::{BootCommand, KernelEdit};
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::rows::SWITCH_WIDTH;

const LINES: u8 = 2;

impl ListItem for Row<'_> {
    fn key(&self) -> &str {
        self.kernel.name.as_str()
    }

    fn name(&self) -> &str {
        self.kernel.name.as_str()
    }
}

/// A row's height before it was laid out, in the fonts in use. Kernels
/// with headers or modules grow below it.
pub fn estimate(ui: &Ui) -> f32 {
    row_height(ui, text_block_height(ui, LINES))
}

pub fn show(ui: &mut Ui, context: &PageContext<'_>, row: &Row<'_>, state: &RowState) {
    let name = row.kernel.name.as_str();
    let details = row.kernel.is_installed() && row.queued != Some(Toggle::Off);
    let gap = ui.tokens().sizes.control_gap;
    let right_width = if details {
        let text = i18n::text(ui.ctx(), "kernels-set-default");
        SWITCH_WIDTH + gap + pill_width(ui, &text)
    } else {
        SWITCH_WIDTH
    };

    split_row(
        ui,
        right_width,
        |ui| {
            title(ui, row);
            text(ui, row);

            if details {
                headers(ui, context, row);
                modules(ui, context, row);
            }
        },
        |ui| {
            if details {
                let spacing = ui.spacing().item_spacing.x;
                ui.spacing_mut().item_spacing.x = gap;
                default_link(ui, context, row);
                ui.spacing_mut().item_spacing.x = spacing;
            }

            // The running kernel stays, so its switch only takes back what
            // a restored queue may hold.
            let enabled = !(row.running && row.kernel.is_installed() && row.queued.is_none());
            let enabled = enabled && !context.busy;
            let flipped = switch(ui, row.wanted, name, enabled).clicked();
            if flipped || (state.toggle && enabled) {
                context.commands.send(KernelEdit::Toggle {
                    kernel: row.kernel.name.clone(),
                    on: !row.wanted,
                });
            }
        },
    );
}

fn title(ui: &mut Ui, row: &Row<'_>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    toolbox_gui_kit::widgets::title_line(ui, true, |ui| {
        let name = row.kernel.name.as_str();
        ui.label(rich(ui, name, Weight::SemiBold, BODY, tokens.text));

        let state = match row.queued {
            Some(Toggle::On) => Some(("kernels-installs", Tone::Accent)),
            Some(Toggle::Off) => Some(("kernels-removes", Tone::Accent)),
            None => row.running.then_some(("kernels-running", Tone::Success)),
        };

        if let Some((id, tone)) = state {
            status_dot(ui, &i18n::text(&ctx, id), tone);
        }
    });
}

fn text(ui: &mut Ui, row: &Row<'_>) {
    let ctx = ui.ctx().clone();
    let tokens = tokens::current(&ctx);
    let kernel = row.kernel;

    if !kernel.description.is_empty() {
        let description = rich(
            ui,
            &kernel.description,
            Weight::Regular,
            SMALL,
            tokens.secondary,
        );

        ui.add(Label::new(description).wrap());
    }

    let mut version = if kernel.repo.is_empty() {
        i18n::text_with(
            &ctx,
            "kernels-version-local",
            &[("version", &kernel.version)],
        )
    } else {
        i18n::text_with(
            &ctx,
            "kernels-version",
            &[("version", &kernel.version), ("repo", &kernel.repo)],
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
            &[("installed", installed)],
        );

        version = format!("{version} · {note}").into();
    }

    ui.add(Label::new(rich(ui, &version, Weight::Regular, SMALL, tokens.secondary)).wrap());

    if row.running && kernel.is_installed() {
        let note = i18n::text(&ctx, "kernels-running-note");
        ui.add(Label::new(rich(ui, &note, Weight::Regular, SMALL, tokens.secondary)).wrap());
    }
}

fn headers(ui: &mut Ui, context: &PageContext<'_>, row: &Row<'_>) {
    let ctx = ui.ctx().clone();
    let queued = row.headers_queued == Some(Toggle::On);
    let (state, tone, action) = if row.kernel.headers_installed {
        ("kernels-headers-installed", Tone::Neutral, None)
    } else if queued {
        (
            "kernels-headers-queued",
            Tone::Accent,
            Some("kernels-unqueue-headers"),
        )
    } else if row.lacks_headers {
        (
            "kernels-headers-missing",
            Tone::Warning,
            Some("kernels-queue-headers"),
        )
    } else {
        return;
    };

    ui.add_space(ui.tokens().sizes.tight_gap);
    tags_line(ui, action.is_some(), |ui| {
        queued_tag(ui, &i18n::text(&ctx, state), tone);
        if let Some(action) = action
            && waiting_button(ui, context, &i18n::text(&ctx, action)).clicked()
        {
            context.commands.send(KernelEdit::Headers {
                kernel: row.kernel.name.clone(),
                on: !queued,
            });
        }
    });
}

fn modules(ui: &mut Ui, context: &PageContext<'_>, row: &Row<'_>) {
    if row.modules.is_empty() {
        return;
    }

    let ctx = ui.ctx().clone();
    ui.add_space(ui.tokens().sizes.tight_gap);
    tags_line(ui, row.unbuilt() > 0, |ui| {
        for module in &row.modules {
            let (id, tone) = module_text(module);
            let text = i18n::text_with(
                &ctx,
                id,
                &[
                    ("module", module.status.module.as_str()),
                    ("version", module.status.version.as_str()),
                ],
            );

            queued_tag(ui, &text, tone);
        }

        if row.unbuilt() == 0 {
            return;
        }

        let queued = row.builds_queued();
        let action = if queued {
            "kernels-unqueue-builds"
        } else {
            "kernels-queue-builds"
        };

        if waiting_button(ui, context, &i18n::text(&ctx, action)).clicked() {
            context.commands.send(KernelEdit::Builds {
                kernel: row.kernel.name.clone(),
                on: !queued,
            });
        }
    });
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

/// A wrapped line of tags, as tall as a pill button when `with_button`,
/// so the tags centre on it. A line starts at the style's own height and
/// grows only once the taller pill comes, which leaves the tags higher up.
fn tags_line(ui: &mut Ui, with_button: bool, add: impl FnOnce(&mut Ui)) {
    ui.scope(|ui| {
        if with_button {
            ui.spacing_mut().interact_size.y = PILL_HEIGHT;
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

    ui.add_enabled_ui(!context.busy, |ui| pill_button(ui, text, ButtonKind::Soft))
        .inner
}

/// Opens Boot options with this kernel's menu entry offered as the
/// default. Nothing changes until it is taken there.
fn default_link(ui: &mut Ui, context: &PageContext<'_>, row: &Row<'_>) {
    let ctx = ui.ctx().clone();
    let name = row.kernel.name.as_str();
    let text = i18n::text(&ctx, "kernels-set-default");
    let label = i18n::text_with(&ctx, "kernels-set-default-name", &[("kernel", name)]);
    let response = named_pill_button(ui, &text, ButtonKind::Soft, &label);

    if response.clicked() {
        context
            .commands
            .send(BootCommand::Propose(row.kernel.name.clone()));
    }
}
