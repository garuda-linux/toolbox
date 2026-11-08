//! This is all the root helper hands out of `grub.cfg`, which may hold a password
//! hash: titles and ids, never its text.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::protocol::{DecodeError, decode};
use crate::{MenuEntryId, PackageName};

/// Between a submenu and an entry in it, in ids and positions.
const INSIDE: char = '>';

/// The words after which a `menuentry` line names its id. GRUB 2.00
/// configs write `--id`, newer ones the variable that holds it.
const ID_MARKS: [&str; 3] = ["--id", "$menuentry_id_option", "${menuentry_id_option}"];

const MAX_TITLE_CHARS: usize = 512;

/// More entries than any menu shows. grub-btrfs keeps its snapshots in
/// a file of its own, so a real menu has a few dozen.
const MAX_ENTRIES: usize = 1000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuEntry {
    /// What `GRUB_DEFAULT` takes: the entry's own id, after the ids of the
    /// submenus around it (`gnulinux-advanced-…>gnulinux-linux-zen-…`).
    pub id: MenuEntryId,
    /// What the menu shows, without control characters.
    pub title: String,
    /// The numbers `GRUB_DEFAULT` also takes: `0`, or `1>2` in a submenu.
    pub position: String,
    pub submenu: bool,
    pub parent: Option<String>,
}

/// The answer to `garuda-toolbox-helper read grub-menu`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrubMenu {
    pub entries: Vec<MenuEntry>,
}

struct Header {
    submenu: bool,
    title: String,
    id: String,
}

/// A submenu whose `}` has not come yet.
struct Open {
    /// `None` when its id is no [`MenuEntryId`]: nothing in it can be
    /// named then.
    id: Option<String>,
    title: String,
    position: String,
    /// How many braces were open before its own.
    depth: usize,
    children: usize,
}

#[derive(Default)]
struct Walk {
    entries: Vec<MenuEntry>,
    open: Vec<Open>,
    top: usize,
    depth: usize,
}

impl GrubMenu {
    /// Entries whose id is no [`MenuEntryId`] are left out.
    pub fn from_config(config: &str) -> Self {
        let mut walk = Walk::default();

        for line in config.lines().map(str::trim) {
            walk.line(line);
        }

        Self {
            entries: walk.entries,
        }
    }

    /// Ids are checked while decoding, and a menu longer than any real one is cut.
    ///
    /// # Errors
    ///
    /// [`DecodeError`] for anything but a menu.
    pub fn parse(line: &str) -> Result<Self, DecodeError> {
        let mut menu: Self = decode(line)?;

        menu.entries.truncate(MAX_ENTRIES);
        Ok(menu)
    }

    /// The entry `GRUB_DEFAULT`'s value names: by id, by position, by
    /// title, or by the id inside its submenu alone.
    pub fn find(&self, default: &str) -> Option<&MenuEntry> {
        let inner = format!("{INSIDE}{default}");

        self.entries
            .iter()
            .find(|entry| entry.id.as_str() == default || entry.position == default)
            .or_else(|| self.entries.iter().find(|entry| entry.title == default))
            .or_else(|| {
                self.entries
                    .iter()
                    .find(|entry| entry.id.as_str().ends_with(&inner))
            })
    }

    /// The entry that boots the kernel package `kernel` with its normal
    /// initramfs. GRUB's `10_linux` gives it the id
    /// `gnulinux-<kernel>-advanced-<device>`, so `linux` never matches
    /// the entry of `linux-zen`.
    pub fn kernel_entry(&self, kernel: &PackageName) -> Option<&MenuEntry> {
        let start = format!("gnulinux-{kernel}-advanced-");

        self.entries.iter().find(|entry| {
            let own = entry.id.as_str().rsplit(INSIDE).next().unwrap_or_default();

            !entry.submenu && own.starts_with(&start)
        })
    }
}

impl Walk {
    fn line(&mut self, line: &str) {
        let opens = line.ends_with('{');

        if let Some(header) = header(line) {
            self.entry(&header, opens);
        }

        if opens {
            self.depth = self.depth.saturating_add(1);
        } else if line == "}" {
            self.depth = self.depth.saturating_sub(1);

            while self
                .open
                .last()
                .is_some_and(|submenu| submenu.depth >= self.depth)
            {
                self.open.pop();
            }
        } else {
            // Other lines open and close nothing, so the chain needs no action here.
        }
    }

    fn entry(&mut self, header: &Header, opens: bool) {
        let (number, around, places, parent) = if let Some(submenu) = self.open.last_mut() {
            let number = submenu.children;

            submenu.children = number.saturating_add(1);
            (
                number,
                submenu.id.as_ref().map(|id| format!("{id}{INSIDE}")),
                format!("{}{INSIDE}", submenu.position),
                Some(submenu.title.clone()),
            )
        } else {
            let number = self.top;

            self.top = number.saturating_add(1);
            (number, Some(String::new()), String::new(), None)
        };

        let position = format!("{places}{number}");
        let title = shown(&header.title);
        let id = around
            .map(|ids| format!("{ids}{}", header.id))
            .and_then(|full| MenuEntryId::new(full).ok());

        if let Some(id) = &id
            && self.entries.len() < MAX_ENTRIES
        {
            self.entries.push(MenuEntry {
                id: id.clone(),
                title: title.clone(),
                position: position.clone(),
                submenu: header.submenu,
                parent,
            });
        }

        if header.submenu && opens {
            self.open.push(Open {
                id: id.map(|id| id.as_str().to_owned()),
                title,
                position,
                depth: self.depth,
                children: 0,
            });
        }
    }
}

/// Without an id of its own an entry goes by its title.
fn header(line: &str) -> Option<Header> {
    let (submenu, rest) = line
        .strip_prefix("menuentry")
        .map(|rest| (false, rest))
        .or_else(|| line.strip_prefix("submenu").map(|rest| (true, rest)))?;

    if !rest.starts_with([' ', '\t']) {
        return None;
    }

    let mut words = words(rest).into_iter();
    let title = words.next()?;
    let mut id = None;

    while let Some(word) = words.next() {
        if ID_MARKS.contains(&word.as_str()) {
            id = words.next().filter(|next| next != "{");
        } else if let Some(inline) = word.strip_prefix("--id=") {
            id = Some(inline.to_owned());
        } else {
            // Classes, users and hotkeys are no part of the menu shown.
        }
    }

    Some(Header {
        submenu,
        id: id.unwrap_or_else(|| title.clone()),
        title,
    })
}

/// A title as the page shows it: one line of bounded length.
fn shown(title: &str) -> String {
    title
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_TITLE_CHARS)
        .collect()
}

/// Splits a line the way GRUB's script reader does: blanks part the
/// words, quotes keep them together, `'It'\''s'` is one word.
fn words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word: Option<String> = None;
    let mut characters = text.chars();

    while let Some(character) = characters.next() {
        if character.is_whitespace() {
            words.extend(word.take());
            continue;
        }

        let current = word.get_or_insert_with(String::new);

        match character {
            '\'' => current.extend(characters.by_ref().take_while(|inner| *inner != '\'')),
            '"' => {
                while let Some(inner) = characters.next().filter(|inner| *inner != '"') {
                    if inner == '\\' {
                        current.extend(characters.next());
                    } else {
                        current.push(inner);
                    }
                }
            }
            '\\' => current.extend(characters.next()),
            _ => current.push(character),
        }
    }

    words.extend(word);
    words
}
