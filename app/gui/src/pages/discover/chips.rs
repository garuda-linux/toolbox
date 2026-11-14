//! The texts of the category chips, built when the search, the system,
//! the queue or the look change and kept until then.

use toolbox_core::features::discover::Category;

use super::category_title;
use crate::i18n;

#[derive(Debug, Default)]
pub struct Chips {
    /// The title of the chip for every category.
    pub all: String,
    /// How many apps every category has together.
    pub total: String,
    pub categories: Vec<CategoryChip>,
}

#[derive(Debug)]
pub struct CategoryChip {
    pub id: String,
    pub title: String,
    pub count: String,
}

impl Chips {
    pub fn of(ctx: &egui::Context, categories: &[Category<'_>]) -> Self {
        let total: usize = categories.iter().map(|category| category.count).sum();
        let categories = categories
            .iter()
            .map(|category| {
                let id = category.section.id();
                CategoryChip {
                    id: id.to_owned(),
                    title: category_title(ctx, id, category.section.title()),
                    count: category.count.to_string(),
                }
            })
            .collect();

        Self {
            all: i18n::text(ctx, "discover-all").into_owned(),
            total: total.to_string(),
            categories,
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.categories.iter().map(|chip| chip.id.as_str())
    }
}
