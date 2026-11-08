use super::Guide;

const SUMMARY: &str = "# Summary\n\n[Introduction](introduction.md)\n\n# System\n\n- [Kernels](kernels.md)\n- [Gone](gone.md)\n- [Network](network.md)\n";

fn sources() -> Vec<(&'static str, &'static str)> {
    vec![
        ("introduction.md", "# Introduction\n\nWelcome to the app.\n"),
        (
            "kernels.md",
            "# Kernels\n\nLists kernels.\n\n## Install a kernel\n\nTurn on the switch of `linux-lts`.\n\n### Headers\n\nThe headers come along.\n",
        ),
        (
            "network.md",
            "# Network\n\nShows the interfaces and the kernel driver.\n",
        ),
        ("unlisted.md", "# Unlisted\n"),
    ]
}

#[test]
fn pages_follow_the_summary_and_know_their_part() {
    let guide = Guide::new(SUMMARY, &sources());

    let files: Vec<_> = guide
        .pages()
        .iter()
        .map(|page| page.file.as_str())
        .collect();

    assert_eq!(files, ["introduction.md", "kernels.md", "network.md"]);
    assert_eq!(guide.pages()[1].part.as_deref(), Some("System"));

    assert_eq!(guide.pages()[0].part, None);
    assert_eq!(guide.parts()[1].title.as_deref(), Some("System"));
    assert_eq!(guide.parts()[1].pages, 1..3);
}

#[test]
fn a_summary_entry_without_its_file_is_reported_and_left_out() {
    let guide = Guide::new(SUMMARY, &sources());

    assert_eq!(guide.missing(), ["gone.md"]);
    assert!(guide.page("gone.md").is_none());
}

#[test]
fn the_outline_lists_the_headings_below_the_title() {
    let guide = Guide::new(SUMMARY, &sources());
    let kernels = guide.page("kernels.md").expect("kernels is listed");

    let outline: Vec<_> = kernels
        .outline
        .iter()
        .map(|entry| (entry.level, entry.anchor.as_str(), entry.text.as_str()))
        .collect();

    assert_eq!(
        outline,
        [
            (2, "install-a-kernel", "Install a kernel"),
            (3, "headers", "Headers")
        ]
    );
}

#[test]
fn previous_and_next_follow_the_reading_order() {
    let guide = Guide::new(SUMMARY, &sources());

    let title = |page: Option<&super::Page>| page.map(|page| page.title.clone());
    assert_eq!(title(guide.previous("introduction.md")), None);
    assert_eq!(
        title(guide.next("introduction.md")).as_deref(),
        Some("Kernels")
    );
    assert_eq!(
        title(guide.previous("network.md")).as_deref(),
        Some("Kernels")
    );
    assert_eq!(title(guide.next("network.md")), None);
}

#[test]
fn search_finds_a_heading_and_points_at_it() {
    let guide = Guide::new(SUMMARY, &sources());

    let hits = guide.search("LTS switch");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].file, "kernels.md");
    assert_eq!(hits[0].anchor.as_deref(), Some("install-a-kernel"));
    assert_eq!(hits[0].heading.as_deref(), Some("Install a kernel"));
    assert!(hits[0].snippet.contains("linux-lts"));
}

#[test]
fn search_ranks_a_page_title_above_a_word_in_the_text() {
    let guide = Guide::new(SUMMARY, &sources());

    let hits = guide.search("kernel");
    let places: Vec<_> = hits
        .iter()
        .map(|hit| (hit.file.as_str(), hit.anchor.as_deref()))
        .collect();

    assert_eq!(
        places,
        [
            ("kernels.md", None),
            ("kernels.md", Some("install-a-kernel")),
            ("network.md", None),
        ]
    );
}

#[test]
fn an_empty_search_finds_nothing() {
    let guide = Guide::new(SUMMARY, &sources());

    assert_eq!(guide.search("  "), []);
    assert_eq!(guide.search("zzz"), []);
}
