//! The window's answers to commands that ask about its queue over the
//! session bus.

#[cfg(test)]
mod tests;

use toolbox_core::{Snapshot, domains};
use toolbox_instance::bus::queue::{Question, Refusal};

use super::Queue;
use super::edit::QueueEdit;

impl Queue {
    /// Answers `question`. `busy` while the Console runs a plan: a change
    /// queued then could be dropped as done once the plan succeeds.
    ///
    /// # Errors
    ///
    /// [`Refusal::Busy`] for an edit while `busy`, [`Refusal::Invalid`]
    /// for one that does not parse.
    pub fn answer(
        &mut self,
        question: Question,
        busy: bool,
        snapshot: &Snapshot,
    ) -> Result<String, Refusal> {
        match question {
            Question::Changes => serde_json::to_string(&self.changes.to_saved()).map_err(|error| {
                tracing::warn!("the queue could not be told: {error}");
                Refusal::Invalid
            }),
            Question::Edit(_) if busy => Err(Refusal::Busy),
            Question::Edit(text) => {
                let domains = domains::all().map_err(|error| {
                    tracing::error!("the change domains are broken: {error}");
                    Refusal::Invalid
                })?;

                let edit = QueueEdit::parse(&text, &domains).map_err(|invalid| {
                    tracing::warn!("refused a queue edit from a command: {}", invalid.as_str());
                    Refusal::Invalid
                })?;

                tracing::info!(
                    "a command takes back {} and queues {} changes",
                    edit.remove.len(),
                    edit.add.len()
                );
                self.take(&edit, &domains, snapshot);
                Ok(String::new())
            }
        }
    }
}
