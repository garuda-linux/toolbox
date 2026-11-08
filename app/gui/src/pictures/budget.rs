//! How many app pictures stay decoded. egui keeps every image it loaded,
//! and scrolling through all of Discover's apps would keep about 1,700.
//! The pictures drawn longest ago leave once there are more than the
//! limit, and load again when their rows come back.

#[cfg(test)]
mod tests;

use std::collections::VecDeque;

/// The pictures drawn, the one drawn longest ago first.
#[derive(Debug)]
pub struct Budget {
    limit: usize,
    drawn: VecDeque<String>,
}

impl Budget {
    pub const fn new(limit: usize) -> Self {
        Self {
            limit,
            drawn: VecDeque::new(),
        }
    }

    /// Notes that `uri` was drawn. Returns the picture that leaves for it.
    pub fn drawn(&mut self, uri: &str) -> Option<String> {
        if let Some(at) = self.drawn.iter().position(|seen| seen == uri) {
            let seen = self.drawn.remove(at)?;
            self.drawn.push_back(seen);
            return None;
        }

        self.drawn.push_back(uri.to_owned());
        if self.drawn.len() > self.limit {
            return self.drawn.pop_front();
        }

        None
    }
}
