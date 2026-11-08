//! What a later start asks of the window besides a page, so the command
//! line can name it without the window's bus side.

use toolbox_core::features::diagnostics::Selection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageAction {
    Collect(Selection),
}
