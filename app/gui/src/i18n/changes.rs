#[cfg(test)]
mod tests;

use egui::Context;
use toolbox_core::Summary;
pub use toolbox_services::i18n::changes::sentence_in;

use super::language;

pub fn sentence(ctx: &Context, summary: &Summary) -> String {
    sentence_in(&language(ctx), summary)
}
