//! The links between the guide's pages. Each page and heading a link
//! names must exist, so a reader never lands on nothing. The tests and
//! `cargo xtask guide-links` both read this check.

use crate::{Block, Guide, Link, Page};

/// One line per link that finds no page or heading. Empty when every
/// link does.
pub fn broken_links(guide: &Guide) -> Vec<String> {
    let mut broken = Vec::new();

    for page in guide.pages() {
        for block in all_blocks(page) {
            for link in block
                .spans()
                .into_iter()
                .filter_map(|span| span.link.as_ref())
            {
                if !found(guide, page, link) {
                    broken.push(format!("{}: {link:?}", page.file));
                }
            }
        }
    }

    broken
}

fn found(guide: &Guide, page: &Page, link: &Link) -> bool {
    match link {
        Link::Page { file, anchor } => guide.page(file).is_some_and(|target| {
            anchor
                .as_ref()
                .is_none_or(|anchor| has_anchor(target, anchor))
        }),
        Link::Anchor(anchor) => has_anchor(page, anchor),
        Link::Web(_) => true,
    }
}

/// Every block of `page`, its children included, in reading order.
pub(crate) fn all_blocks(page: &Page) -> Vec<&Block> {
    fn walk<'page>(blocks: &'page [Block], found: &mut Vec<&'page Block>) {
        for block in blocks {
            found.push(block);
            for child in block.children() {
                walk(std::slice::from_ref(child), found);
            }
        }
    }

    let mut found = Vec::new();
    walk(&page.blocks, &mut found);
    found
}

fn has_anchor(page: &Page, anchor: &str) -> bool {
    page.blocks
        .iter()
        .filter_map(Block::heading)
        .any(|heading| heading.anchor == anchor)
}
