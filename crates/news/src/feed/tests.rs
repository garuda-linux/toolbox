use super::{MAX_POSTS, TITLE_CHARS, parse};
use crate::Source;

/// The forum's announcements feed: 25 posts, titles with emoji, CDATA
/// descriptions, categories `Announcements` and `ISO Releases`.
const ANNOUNCEMENTS: &str = include_str!("fixtures/garuda-announcements.rss");
/// The forum's maintenance feed: 7 posts, all of the category
/// `Announcements - Maintenance`.
const MAINTENANCE: &str = include_str!("fixtures/garuda-maintenance.rss");
/// archlinux.org's news feed: 10 posts, three of them asking for manual
/// intervention.
const ARCH: &str = include_str!("fixtures/arch-news.rss");

fn titles(feed: &str, source: Source) -> Vec<String> {
    parse(feed.as_bytes(), source)
        .unwrap()
        .into_iter()
        .map(|post| post.title)
        .collect()
}

#[test]
fn every_item_gives_its_title_link_and_date() {
    let posts = parse(ANNOUNCEMENTS.as_bytes(), Source::Garuda).unwrap();

    assert_eq!(posts[0].title, "Chaotic-AUR turns 8 years old today! 🎉");
    assert_eq!(
        posts[0].link.as_deref(),
        Some("https://forum.garudalinux.org/t/chaotic-aur-turns-8-years-old-today/49226")
    );
    assert_eq!(posts[0].published, Some(1_792_327_265));
    assert_eq!(posts[1].title, "Testing backups the hard way");

    let maintenance = titles(MAINTENANCE, Source::Garuda);

    assert_eq!(
        maintenance[0],
        "Patch for Linux 6.15's BTRFS regression available"
    );
    assert!(
        maintenance[1].contains("Garuda Settings Manager & MHWD"),
        "{maintenance:?}"
    );
}

#[test]
fn the_channel_title_is_no_post() {
    for (feed, source) in [
        (ANNOUNCEMENTS, Source::Garuda),
        (MAINTENANCE, Source::Garuda),
        (ARCH, Source::Arch),
    ] {
        let titles = titles(feed, source);

        assert!(
            titles
                .iter()
                .all(|title| !title.ends_with("Garuda Linux Forum")
                    && title != "Arch Linux: Recent news updates"),
            "{titles:?}"
        );
    }
}

#[test]
fn a_link_away_from_the_forum_is_dropped() {
    let feed = "<rss><channel><item><title>Look</title>\
                <link>javascript:alert(1)</link></item>\
                <item><title>Other</title><link>https://example.org/x</link></item>\
                </channel></rss>";

    let posts = parse(feed.as_bytes(), Source::Garuda).unwrap();

    assert_eq!(posts[0].link, None);
    assert_eq!(posts[1].link, None);
}

#[test]
fn hidden_characters_are_dropped_and_long_titles_cut() {
    let long = "x".repeat(TITLE_CHARS + 50);
    let feed = format!(
        "<rss><channel><item><title>a\u{202e}b&#7;c</title></item>\
         <item><title>{long}</title></item></channel></rss>"
    );

    let posts = parse(feed.as_bytes(), Source::Garuda).unwrap();

    assert_eq!(posts[0].title, "abc");
    assert_eq!(posts[1].title.chars().count(), TITLE_CHARS);
}

#[test]
fn an_item_without_a_title_is_left_out_and_a_bad_date_is_none() {
    // The maintenance feed edited: the first post's title cut, the
    // second post's date made a word.
    let feed = MAINTENANCE
        .replacen(
            "<title>Patch for Linux 6.15&#39;s BTRFS regression available</title>",
            "",
            1,
        )
        .replacen(
            "<pubDate>Sat, 19 Jul 2025 17:40:13 +0000</pubDate>",
            "<pubDate>yesterday</pubDate>",
            1,
        );

    assert_ne!(feed, MAINTENANCE);

    let posts = parse(feed.as_bytes(), Source::Garuda).unwrap();

    assert_eq!(posts.len(), 6);
    assert!(
        posts[0]
            .title
            .starts_with("Modern Garuda Linux hardware profiles")
    );
    assert_eq!(posts[0].published, None);
}

#[test]
fn only_the_newest_posts_are_kept() {
    let posts = parse(ANNOUNCEMENTS.as_bytes(), Source::Garuda).unwrap();

    assert_eq!(posts.len(), MAX_POSTS);
    assert!(ANNOUNCEMENTS.matches("<item>").count() > MAX_POSTS);
}

#[test]
fn broken_xml_is_an_error() {
    parse(b"<rss><channel><item><title>a</item></rss>", Source::Garuda).unwrap_err();
}

#[test]
fn a_garuda_maintenance_post_asks_for_a_step() {
    let maintenance = parse(MAINTENANCE.as_bytes(), Source::Garuda).unwrap();
    let announcements = parse(ANNOUNCEMENTS.as_bytes(), Source::Garuda).unwrap();

    assert!(maintenance.iter().all(|post| post.manual));
    assert!(announcements.iter().all(|post| !post.manual));
    assert_eq!(maintenance[0].source, Source::Garuda);
}

#[test]
fn an_arch_post_asks_for_a_step_by_its_title() {
    let posts = parse(ARCH.as_bytes(), Source::Arch).unwrap();
    let manual: Vec<&str> = posts
        .iter()
        .filter(|post| post.manual)
        .map(|post| post.title.as_str())
        .collect();

    assert_eq!(
        manual,
        [
            "Mkinitcpio >=42 requires manual intervention for TPM2-based unlocking of LUKS devices",
            "virtualbox-ext-vnc >= 7.2.12-2 requires manual intervention",
            "kea >= 1:3.0.3-6 update requires manual intervention",
        ]
    );
    assert!(posts.iter().all(|post| {
        post.link
            .as_deref()
            .is_some_and(|link| link.starts_with("https://archlinux.org/news/"))
    }));

    // The first post's link edited to point at the Garuda forum.
    let elsewhere = ARCH.replacen(
        "<link>https://archlinux.org/news/update-to-php",
        "<link>https://forum.garudalinux.org/t/update-to-php",
        1,
    );

    let moved = parse(elsewhere.as_bytes(), Source::Arch).unwrap();

    assert_eq!(moved[0].link, None, "an Arch post links to Arch only");
}
