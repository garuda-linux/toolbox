//! Row menus as actions.
//!
//! Each row kind lists its actions in sections, and an action says its
//! label, its icon, whether it can run now (and why not, shown on hover)
//! and what it sends. The actions every kind may offer are written once
//! here.

use egui::{Context, Ui};
use toolbox_gui_kit::icons::{self, Icon};
use toolbox_gui_kit::toasts::ToastKind;
use toolbox_gui_kit::widgets::{menu_action, menu_heading};

use crate::command::{AppCommand, CommandSender};
use crate::i18n;

#[derive(Clone, Copy)]
pub struct Run<'a> {
    pub ctx: &'a Context,
    pub commands: &'a CommandSender,
}

pub trait RowAction<Item: ?Sized> {
    /// The text id of the label.
    fn label(&self, item: &Item) -> &'static str;

    fn icon(&self, _item: &Item) -> Option<Icon> {
        None
    }

    fn offered(&self, _item: &Item) -> bool {
        true
    }

    /// # Errors
    ///
    /// The text id of why not, shown on hover.
    fn possible(&self, _item: &Item) -> Result<(), &'static str> {
        Ok(())
    }

    fn run(&self, item: &Item, run: Run<'_>);
}

pub struct Section<Item: ?Sized + 'static> {
    pub heading: Option<&'static str>,
    pub actions: &'static [&'static dyn RowAction<Item>],
}

pub trait RowKind {
    type Item: ?Sized + 'static;

    const SECTIONS: &'static [Section<Self::Item>];
}

pub trait Named {
    fn copied_name(&self) -> &str;
}

pub trait Website {
    fn website(&self) -> Option<&str>;
}

pub struct OpenWebsite;

pub struct CopyName;

impl<T: Website + ?Sized> RowAction<T> for OpenWebsite {
    fn label(&self, _item: &T) -> &'static str {
        "row-open-website"
    }

    fn icon(&self, _item: &T) -> Option<Icon> {
        Some(icons::GLOBE)
    }

    fn offered(&self, item: &T) -> bool {
        item.website().is_some()
    }

    fn run(&self, item: &T, run: Run<'_>) {
        if let Some(url) = item.website() {
            crate::links::open(run.ctx, url);
        }
    }
}

impl<T: Named + ?Sized> RowAction<T> for CopyName {
    fn label(&self, _item: &T) -> &'static str {
        "row-copy-name"
    }

    fn icon(&self, _item: &T) -> Option<Icon> {
        Some(icons::EDIT_COPY)
    }

    fn run(&self, item: &T, run: Run<'_>) {
        let name = item.copied_name();
        run.ctx.copy_text(name.to_owned());
        let text = i18n::text_with(run.ctx, "toast-copied", &[("name", name)]);
        run.commands.send(AppCommand::Toast {
            kind: ToastKind::Info,
            text: text.into_owned(),
        });
    }
}

/// The menu of `item`: each section's offered actions, the ones that
/// cannot run now resting with their reason on hover.
pub fn menu<K>(ui: &mut Ui, item: &K::Item, commands: &CommandSender)
where
    K: RowKind,
{
    let ctx = ui.ctx().clone();
    let run = Run {
        ctx: &ctx,
        commands,
    };

    for section in K::SECTIONS {
        let mut offered = section
            .actions
            .iter()
            .filter(|action| action.offered(item))
            .peekable();

        if offered.peek().is_none() {
            continue;
        }

        if let Some(heading) = section.heading {
            menu_heading(ui, &i18n::text(&ctx, heading));
        }

        for action in offered {
            let possible = action.possible(item);
            let label = i18n::text(&ctx, action.label(item));
            let response = ui
                .add_enabled_ui(possible.is_ok(), |ui| {
                    menu_action(ui, action.icon(item), &label)
                })
                .inner;

            let response = match possible {
                Ok(()) => response,
                Err(why) => response.on_disabled_hover_text(i18n::text(&ctx, why)),
            };

            if response.clicked() {
                action.run(item, run);
            }
        }
    }
}
