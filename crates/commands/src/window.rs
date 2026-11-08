//! The window's start options as this app's pages and page actions.
//! Checked before the window or the bus are touched, so a typo ends with
//! a message instead of a window on Home.

#[cfg(test)]
mod tests;

use toolbox_cli::args::Window;
use toolbox_core::features::diagnostics::Selection;
use toolbox_instance::activation::Activation;
use toolbox_plan::PageName;
use toolbox_services::instance::PageAction;
use toolbox_services::page::PageId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Opening {
    pub page: Option<PageId>,
    pub action: Option<PageAction>,
}

impl Opening {
    /// The wish as a running window takes it over the session bus.
    pub fn activation(&self) -> Activation {
        Activation {
            page: self.page.and_then(|page| PageName::new(page.slug()).ok()),
            action: self
                .action
                .and_then(|PageAction::Collect(selection)| PageName::new(selection.as_str()).ok()),
            ..Activation::default()
        }
    }
}

/// A page or an action this app does not have. The second line of the
/// page's message is its hint, as `error_text` prints it.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum OpeningError {
    #[error("`{0}` is no page\n`garuda-toolbox pages` lists the page ids")]
    NoPage(PageName),
    #[error("`{0}` is no action of that page")]
    NoAction(PageName),
}

/// # Errors
///
/// [`OpeningError`] for a page or an action this app does not have.
pub fn opening(window: &Window) -> Result<Opening, OpeningError> {
    let page = if window.setup_assistant {
        Some(PageId::Setup)
    } else {
        window
            .page
            .as_ref()
            .map(|id| {
                PageId::from_slug(id.as_ref()).ok_or_else(|| OpeningError::NoPage(id.clone()))
            })
            .transpose()?
    };

    let action = window
        .action
        .as_ref()
        .map(|id| match (page, Selection::from_id(id.as_ref())) {
            (Some(PageId::Diagnostics), Some(selection)) => Ok(PageAction::Collect(selection)),
            _ => Err(OpeningError::NoAction(id.clone())),
        })
        .transpose()?;

    Ok(Opening { page, action })
}
