use egui::{Key, Label, TextEdit, Ui};
use toolbox_core::domains::SdBootKernel;
use toolbox_core::features::boot::edit::Edit;
use toolbox_core::features::boot::view::{self, Page};
use toolbox_core::features::boot::{Flag, kernel};
use toolbox_gui_kit::fonts::{BODY, Weight};
use toolbox_gui_kit::widgets::{
    ButtonKind, Tone, card, card_row, pill_button, removable_tag, rich, tag, text_field,
};
use toolbox_gui_kit::{UiExt as _, icons};

use super::menu::flipped;
use super::{heading, queued, send};
use crate::i18n;
use crate::pages::PageContext;
use crate::pages::setting_row::{Words, row};

const FIELD_WIDTH: f32 = 240.0;

pub fn show(
    ui: &mut Ui,
    context: &PageContext<'_>,
    page: &Page,
    typed: &mut String,
) -> Option<String> {
    let ctx = ui.ctx().clone();
    let sdboot = kernel(context.snapshot);
    heading(ui, icons::UTILITIES_TERMINAL, "boot-kernel");
    if !page.parameters_editable() {
        let line = view::parameters_line(context.snapshot);
        let id = if sdboot.is_some() {
            "boot-sdboot-parameters-custom"
        } else {
            "boot-parameters-custom"
        };

        let text = i18n::text_with(&ctx, id, &[("line", &line)]);
        card(ui, |ui| {
            card_row(ui, true, |ui| {
                let secondary = ui.tokens().secondary;
                ui.add(Label::new(rich(ui, &text, Weight::Regular, BODY, secondary)).wrap());
            });
        });

        return None;
    }

    card(ui, |ui| {
        let added = parameters(ui, context, page, sdboot, typed);
        let queued_flags = page.flags_queued();
        for flag in Flag::ALL {
            let on = page.flag(flag);
            let tag = queued(&ctx, queued_flags.contains(&flag.as_str()));
            let id = format!("boot-flag-{}", flag.as_str());
            if flipped(ui, &id, tag.as_deref(), on) {
                send(context, Edit::Flag(flag, !on));
            }
        }

        added
    })
    .inner
}

fn parameters(
    ui: &mut Ui,
    context: &PageContext<'_>,
    page: &Page,
    sdboot: Option<&SdBootKernel>,
    typed: &mut String,
) -> Option<String> {
    let ctx = ui.ctx().clone();
    let title = i18n::text(&ctx, "boot-parameters");
    let description = i18n::text(
        &ctx,
        if sdboot.is_some() {
            "boot-sdboot-parameters-detail"
        } else {
            "boot-parameters-detail"
        },
    );

    let note = sdboot
        .filter(|kernel| kernel.cmdline.running)
        .map(|_| i18n::text(&ctx, "boot-sdboot-cmdline-running"));

    let tag_text = queued(&ctx, view::parameters_queued(context.changes));
    let words = Words {
        current: None,
        title: &title,
        description: &description,
        note: note.as_deref(),
        queued: tag_text.as_deref(),
    };

    let mut added = None;
    row(ui, true, &words, 0.0, |_| {});

    // The tags belong to the words above, so no line parts them.
    card_row(ui, true, |ui| {
        ui.horizontal_wrapped(|ui| chips(ui, context, page));
        ui.add_space(ui.tokens().sizes.card_gap);

        ui.horizontal(|ui| {
            let name = i18n::text(&ctx, "boot-parameter-new").into_owned();
            let field = text_field(
                ui,
                TextEdit::singleline(typed)
                    .hint_text(i18n::text(&ctx, "boot-parameter-hint"))
                    .desired_width(FIELD_WIDTH)
                    .align(egui::Align2::LEFT_CENTER),
            );

            ui.ctx()
                .accesskit_node_builder(field.id, |node| node.set_label(name));

            let entered = field.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
            let add = i18n::text(&ctx, "boot-parameter-add");
            let clicked = pill_button(ui, &add, ButtonKind::Soft).clicked();
            if (entered || clicked) && !typed.trim().is_empty() {
                added = Some(typed.trim().to_owned());
            }
        });
    });

    added
}

fn chips(ui: &mut Ui, context: &PageContext<'_>, page: &Page) {
    let ctx = ui.ctx().clone();
    let items = page.parameter_items();

    if items.is_empty() {
        ui.label(i18n::text(&ctx, "boot-parameters-none"));
    }

    for item in items {
        if !page.removable(item) {
            let kept = i18n::text_with(&ctx, "boot-parameter-kept", &[("parameter", item)]);

            tag(ui, item, Tone::Neutral).on_hover_text(kept);
            continue;
        }

        let remove = i18n::text_with(&ctx, "boot-parameter-remove", &[("parameter", item)]);
        if removable_tag(ui, item, &remove) {
            send(context, Edit::RemoveParameter(item.clone()));
        }
    }
}
