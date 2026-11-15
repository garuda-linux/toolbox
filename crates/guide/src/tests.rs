//! The guide in `docs/src` as built in: every page listed, every link and
//! picture found. A broken link fails here before a reader finds it.

use super::links::{all_blocks, broken_links};
use super::{Block, built_in, embedded, image, image_size};

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
    assert_eq!(
        broken_links(embedded()),
        Vec::<String>::new(),
        "broken links"
    );
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
