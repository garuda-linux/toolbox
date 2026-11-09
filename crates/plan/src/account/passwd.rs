//! The one reading of `/etc/passwd`, so the helper finds every account the
//! app lists.

#[cfg(test)]
mod tests;

use crate::lines::entries;

/// One line of `/etc/passwd`, `name:password:uid:gid:comment:home:shell`,
/// its text fields as the file has them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasswdEntry<'file> {
    pub name: &'file str,
    pub uid: u32,
    pub gid: u32,
    pub gecos: &'file str,
    pub home: &'file str,
    /// Empty when the line has none, which login reads as `/bin/sh`.
    pub shell: &'file str,
}

impl<'file> PasswdEntry<'file> {
    /// `None` when an id is missing or no number.
    pub fn parse(line: &'file str) -> Option<Self> {
        let mut fields = line.split(':');
        let name = fields.next()?;
        let uid = fields.nth(1)?.trim().parse().ok()?;
        let gid = fields.next()?.trim().parse().ok()?;

        Some(Self {
            name,
            uid,
            gid,
            gecos: fields.next().unwrap_or_default(),
            home: fields.next().unwrap_or_default(),
            shell: fields.next().unwrap_or_default().trim(),
        })
    }
}

/// Every entry of `file` that parses, in file order.
pub fn passwd_entries(file: &str) -> impl Iterator<Item = PasswdEntry<'_>> {
    entries(file).filter_map(PasswdEntry::parse)
}
