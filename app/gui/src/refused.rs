//! A page's last refused click: why, and what it was about. It stays on
//! the page until dismissed or a later click on that page is queued.

#[cfg(test)]
mod tests;

use toolbox_core::ChangeSet;
pub use toolbox_core::features::Refused;
use toolbox_core::features::{RefusalText, protection_text_id};
use toolbox_plan::PackageName;
use toolbox_services::queue::clicked::on_copy;

use crate::i18n;
use crate::page_service::Handled;

/// The text of a refusal, in the window's language.
pub trait Shown {
    fn text(&self, ctx: &egui::Context) -> String {
        self.text_with(ctx, &[])
    }

    /// With the placeables a text needs beside `$name`, such as the
    /// `$why` of a protected package.
    fn text_with<'a>(&'a self, ctx: &egui::Context, more: &[(&'static str, &'a str)]) -> String;
}

/// What a text names as `$name`. `()` names nothing, as the texts of
/// Boot Repair.
pub trait Subject {
    fn name(&self) -> Option<&str>;
}

impl Subject for String {
    fn name(&self) -> Option<&str> {
        Some(self)
    }
}

impl Subject for PackageName {
    fn name(&self) -> Option<&str> {
        Some(self.as_str())
    }
}

impl Subject for () {
    fn name(&self) -> Option<&str> {
        None
    }
}

/// A protected package's text also names why it stays, as `$why`.
impl<R: RefusalText, S: Subject> Shown for Refused<R, S> {
    fn text_with<'a>(&'a self, ctx: &egui::Context, more: &[(&'static str, &'a str)]) -> String {
        let why = self
            .refusal
            .protection()
            .map(|why| i18n::text(ctx, &protection_text_id(why)));

        let mut args: Vec<_> = self
            .subject
            .name()
            .map(|name| ("name", name))
            .into_iter()
            .chain(why.as_deref().map(|why| ("why", why)))
            .collect();

        args.extend_from_slice(more);
        i18n::text_with(ctx, &self.refusal.text_id(), &args).into_owned()
    }
}

/// The shape of most clicks: `click` against a copy of `changes`. Taken,
/// its edit goes to the queue and the page's refusal goes. Refused, the
/// queue stays and `slot` names why and `subject`.
pub fn queue_or_refuse<R, S>(
    slot: &mut Option<Refused<R, S>>,
    subject: S,
    changes: &ChangeSet,
    click: impl FnOnce(&mut ChangeSet) -> Result<(), R>,
) -> Handled {
    queue_or_keep(slot, changes, |changes| {
        click(changes).map_err(|refusal| Refused::new(refusal, subject))
    })
}

pub fn queue(changes: &ChangeSet, click: impl FnOnce(&mut ChangeSet)) -> Handled {
    Handled::edit(on_copy(changes, click).0)
}

/// [`queue_or_refuse`] for a rule that names its own subject, such as a
/// protected dependency a removal would take along.
pub fn queue_or_keep<R, S>(
    slot: &mut Option<Refused<R, S>>,
    changes: &ChangeSet,
    click: impl FnOnce(&mut ChangeSet) -> Result<(), Refused<R, S>>,
) -> Handled {
    let (edit, result) = on_copy(changes, click);
    match result {
        Ok(()) => {
            *slot = None;
            Handled::edit(edit)
        }
        Err(refused) => {
            *slot = Some(refused);
            Handled::taken()
        }
    }
}
