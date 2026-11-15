//! What a widget that picks a value answers: its response, as every widget
//! gives, and the value picked this frame, if any.

use egui::Response;

/// The response of a picking widget beside the value it picked.
#[derive(Clone, Debug)]
pub struct Choice<T> {
    pub response: Response,
    pub value: Option<T>,
}
