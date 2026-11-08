use super::{MAX_POSTS, TITLE_CHARS, parse};
use crate::Source;

const FEED: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Announcements - Garuda Linux Forum</title>
    <link>https://forum.garudalinux.org/c/announcements/16</link>
    <item>
      <title>Chaotic-AUR turns 8 years old today! &#127881;</title>
      <description><![CDATA[<p>Today we celebrate.</p>]]></description>
      <link>https://forum.garudalinux.org/t/chaotic-aur-turns-8/48000</link>
      <pubDate>Sun, 18 Oct 2026 12:41:05 +0000</pubDate>
    </item>
    <item>
      <title><![CDATA[Mesa & KDE updates]]></title>
      <link>https://forum.garudalinux.org/t/mesa/47000</link>
      <pubDate>Mon, 05 Oct 2026 08:00:00 +0000</pubDate>
    </item>
  </channel>
</rss>"#;

#[test]
fn every_item_gives_its_title_link_and_date() {
    let posts = parse(FEED.as_bytes(), Source::Garuda).unwrap();

    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].title, "Chaotic-AUR turns 8 years old today! 🎉");
    assert_eq!(
        posts[0].link.as_deref(),
        Some("https://forum.garudalinux.org/t/chaotic-aur-turns-8/48000")
    );
    assert_eq!(posts[0].published, Some(1_792_327_265));
    assert_eq!(posts[1].title, "Mesa & KDE updates");
}

#[test]
fn the_channel_title_is_no_post() {
    let posts = parse(FEED.as_bytes(), Source::Garuda).unwrap();

    assert!(
        posts
            .iter()
            .all(|post| !post.title.contains("Garuda Linux Forum"))
    );
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
    let feed = "<rss><channel><item><link>https://forum.garudalinux.org/t/1</link></item>\
                <item><title>Kept</title><pubDate>yesterday</pubDate></item>\
                </channel></rss>";

    let posts = parse(feed.as_bytes(), Source::Garuda).unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].published, None);
}

#[test]
fn only_the_newest_posts_are_kept() {
    let items = "<item><title>Post</title></item>".repeat(MAX_POSTS + 5);
    let feed = format!("<rss><channel>{items}</channel></rss>");

    assert_eq!(
        parse(feed.as_bytes(), Source::Garuda).unwrap().len(),
        MAX_POSTS
    );
}

#[test]
fn broken_xml_is_an_error() {
    parse(b"<rss><channel><item><title>a</item></rss>", Source::Garuda).unwrap_err();
}

#[test]
fn a_garuda_maintenance_post_asks_for_a_step() {
    let feed = "<rss><channel>\
                <item><title>Patch available</title>\
                <category>Announcements - Maintenance</category></item>\
                <item><title>ISO out</title><category>ISO Releases</category></item>\
                </channel></rss>";

    let posts = parse(feed.as_bytes(), Source::Garuda).unwrap();

    assert!(posts[0].manual);
    assert!(!posts[1].manual);
    assert_eq!(posts[0].source, Source::Garuda);
}

#[test]
fn an_arch_post_asks_for_a_step_by_its_title() {
    let feed = "<rss><channel>\
                <item><title>glibc 2.41 update requires Manual Intervention</title>\
                <link>https://archlinux.org/news/glibc-241/</link></item>\
                <item><title>New Arch Linux mirror policy</title>\
                <link>https://forum.garudalinux.org/t/1</link></item>\
                </channel></rss>";

    let posts = parse(feed.as_bytes(), Source::Arch).unwrap();

    assert!(posts[0].manual);
    assert_eq!(
        posts[0].link.as_deref(),
        Some("https://archlinux.org/news/glibc-241/")
    );
    assert!(!posts[1].manual);
    assert_eq!(posts[1].link, None, "an Arch post links to Arch only");
}
