use super::{Entry, Part, parse};

#[test]
fn prefix_chapters_come_before_the_first_part_without_a_title() {
    let summary = parse("# Summary\n\n[Introduction](introduction.md)\n");

    assert_eq!(
        summary,
        vec![Part {
            title: None,
            entries: vec![Entry {
                title: "Introduction".into(),
                file: "introduction.md".into(),
            }],
        }]
    );
}

#[test]
fn part_headings_group_the_list_entries_under_them() {
    let summary = parse(
        "# Summary\n\n[Introduction](introduction.md)\n\n# Install\n\n- [Gaming](gaming.md)\n- [App Catalog](discover.md)\n\n# Care\n\n* [Maintenance](maintenance.md)\n",
    );

    let titles: Vec<_> = summary.iter().map(|part| part.title.as_deref()).collect();
    assert_eq!(titles, [None, Some("Install"), Some("Care")]);
    assert_eq!(summary[1].entries[1].file, "discover.md");
    assert_eq!(summary[2].entries[0].title, "Maintenance");
}

#[test]
fn a_list_without_parts_lands_in_one_part_without_a_title() {
    let summary = parse("# Summary\n\n- [Home](home.md)\n- [Kernels](kernels.md)\n");

    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].title, None);
    assert_eq!(summary[0].entries.len(), 2);
}

#[test]
fn nested_entries_and_separators_count_as_plain_entries() {
    let summary = parse(
        "# Summary\n\n- [Home](home.md)\n  - [Status](status.md)\n\n---\n\n- [Kernels](./kernels.md)\n",
    );

    let files: Vec<_> = summary[0]
        .entries
        .iter()
        .map(|entry| entry.file.as_str())
        .collect();

    assert_eq!(files, ["home.md", "status.md", "kernels.md"]);
}

#[test]
fn an_empty_part_is_dropped() {
    let summary = parse("# Summary\n\n# Empty\n\n# Care\n\n- [Maintenance](maintenance.md)\n");

    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].title.as_deref(), Some("Care"));
}
