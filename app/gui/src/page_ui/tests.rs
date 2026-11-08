use std::collections::BTreeSet;

use toolbox_gui_kit::tokens::Hue;

use super::{GroupLook as _, PageLook as _, SEARCH_ICON};
use crate::page::PageId;

#[test]
fn pages_have_distinct_icons() {
    let icons: BTreeSet<_> = PageId::ALL.iter().map(|page| page.icon().name()).collect();

    assert_eq!(icons.len(), PageId::ALL.len());
}

#[test]
fn the_search_entry_has_a_glyph_no_page_has() {
    for page in PageId::ALL {
        assert_ne!(
            page.icon(),
            SEARCH_ICON,
            "{page:?} looks like the Search entry"
        );
    }
}

#[test]
fn no_page_looks_like_the_information_glyph() {
    for page in PageId::ALL {
        assert_ne!(
            page.icon(),
            toolbox_gui_kit::icons::HELP_ABOUT,
            "{page:?} looks like the information glyph"
        );
        assert_ne!(
            page.icon().name(),
            "documentinfo",
            "{page:?} uses the circled i of documentinfo"
        );
    }
}

#[test]
fn a_pages_icons_take_the_hue_of_its_sidebar_group() {
    let hue = |page: PageId| page.group().hue();
    assert_eq!(hue(PageId::Gaming), Hue::Blue);
    assert_eq!(hue(PageId::Kernels), Hue::Green);
    assert_eq!(hue(PageId::Maintenance), Hue::Orange);
    for page in [PageId::Home, PageId::Settings, PageId::About] {
        assert_eq!(hue(page), Hue::Accent, "{page:?}");
    }
}
