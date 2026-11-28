//! A file chooser that gives the same answer every time, as a desktop with
//! or without a portal does.

use crate::file_chooser::{Chosen, FileChooser};

#[derive(Clone, Debug)]
pub struct FixedChooser(pub Chosen);

impl FileChooser for FixedChooser {
    fn choose(&self, _title: &str, _filter: &str, _globs: &[&str]) -> Chosen {
        self.0.clone()
    }
}
