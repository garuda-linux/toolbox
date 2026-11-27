use super::Pages;
use crate::command::Reveal;

impl Pages {
    pub fn reveal(&mut self, reveal: &Reveal) {
        match reveal {
            Reveal::Package { name, aur } => self.packages.reveal(name, *aur),
            Reveal::Packages { search, aur } => self.packages.search_for(search, *aur),
        }
    }
}
