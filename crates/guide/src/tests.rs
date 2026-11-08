//! The guide in `docs/src` as built in: every page listed, every link and
//! picture found. A broken link fails here before a reader finds it.

use super::{Block, Link, Page, built_in, embedded, image, image_size};

fn walk<'page>(blocks: &'page [Block], found: &mut Vec<&'page Block>) {
    for block in blocks {
        found.push(block);
        for child in block.children() {
            walk(std::slice::from_ref(child), found);
        }
    }
}

fn all_blocks(page: &Page) -> Vec<&Block> {
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

#[test]
fn every_page_the_summary_names_is_built_in() {
    assert_eq!(embedded().missing(), Vec::<String>::new());
    assert!(embedded().page(super::FIRST_PAGE).is_some());
}

#[test]
fn every_page_in_docs_is_listed_in_the_summary() {
    let unlisted: Vec<_> = built_in::PAGES
        .iter()
        .map(|(file, _)| *file)
        .filter(|file| embedded().page(file).is_none())
        .collect();

    assert_eq!(unlisted, Vec::<&str>::new(), "not in SUMMARY.md");
}

#[test]
fn every_link_between_pages_finds_its_page_and_heading() {
    let guide = embedded();
    let mut broken = Vec::new();

    for page in guide.pages() {
        for block in all_blocks(page) {
            for link in block
                .spans()
                .into_iter()
                .filter_map(|span| span.link.as_ref())
            {
                let found = match link {
                    Link::Page { file, anchor } => guide.page(file).is_some_and(|target| {
                        anchor
                            .as_ref()
                            .is_none_or(|anchor| has_anchor(target, anchor))
                    }),
                    Link::Anchor(anchor) => has_anchor(page, anchor),
                    Link::Web(_) => true,
                };

                if !found {
                    broken.push(format!("{}: {link:?}", page.file));
                }
            }
        }
    }

    assert_eq!(broken, Vec::<String>::new(), "broken links");
}

#[test]
fn every_picture_a_page_shows_is_built_in() {
    let missing: Vec<_> = embedded()
        .pages()
        .iter()
        .flat_map(|page| {
            all_blocks(page).into_iter().filter_map(move |block| {
                let Block::Image(picture) = block else {
                    return None;
                };

                image(&picture.src)
                    .is_none()
                    .then(|| format!("{}: {}", page.file, picture.src))
            })
        })
        .collect();

    assert_eq!(missing, Vec::<String>::new(), "pictures not found");
}

#[test]
fn a_picture_outside_the_guide_is_not_found() {
    assert!(image("images/none.png").is_none());
    assert!(image("../../Cargo.toml").is_none());
}

#[test]
fn every_built_in_picture_tells_its_size_up_front() {
    let unknown: Vec<_> = built_in::IMAGES
        .iter()
        .filter(|(_, bytes)| {
            image_size(bytes).is_none_or(|(width, height)| width == 0 || height == 0)
        })
        .map(|(name, _)| *name)
        .collect();

    assert_eq!(unknown, Vec::<&str>::new(), "pictures without a size");
}

#[test]
fn a_png_header_gives_width_and_height_and_anything_else_nothing() {
    let mut header = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    header.extend(640_u32.to_be_bytes());
    header.extend(480_u32.to_be_bytes());

    assert_eq!(image_size(&header), Some((640, 480)));
    assert_eq!(image_size(b"GIF89a"), None);
    assert_eq!(image_size(&header[..20]), None);
}
