use super::{Block, CalloutKind, Code, Image, Link, Span, Style, parse};

fn plain(text: &str) -> Span {
    Span {
        text: text.into(),
        style: Style::default(),
        link: None,
    }
}

fn paragraph(block: &Block) -> &[Span] {
    let Block::Paragraph(spans) = block else {
        panic!("a paragraph, got {block:?}");
    };

    spans
}

#[test]
fn headings_get_the_anchor_mdbook_gives_them() {
    let blocks = parse("# Kernels\n\n## Boot a kernel by default\n\n## What's new?\n");

    let anchors: Vec<_> = blocks
        .iter()
        .filter_map(Block::heading)
        .map(|heading| (heading.level, heading.anchor.as_str()))
        .collect();

    assert_eq!(
        anchors,
        [
            (1, "kernels"),
            (2, "boot-a-kernel-by-default"),
            (2, "whats-new")
        ]
    );
}

#[test]
fn a_repeated_heading_gets_a_numbered_anchor() {
    let blocks = parse("## Steps\n\n## Steps\n");

    let second = blocks[1].heading().expect("a heading");
    assert_eq!(second.anchor, "steps-1");
}

#[test]
fn inline_styles_and_code_keep_their_text() {
    let blocks = parse("Turn on **Available**, *then* run `paru`.\n");

    let spans = paragraph(&blocks[0]);
    let strong: Vec<_> = spans.iter().filter(|span| span.style.strong).collect();
    let code: Vec<_> = spans.iter().filter(|span| span.style.code).collect();
    assert_eq!(strong[0].text, "Available");
    assert_eq!(code[0].text, "paru");
    assert!(
        spans
            .iter()
            .any(|span| span.style.emphasis && span.text == "then")
    );
}

#[test]
fn links_tell_pages_anchors_and_web_addresses_apart() {
    let blocks = parse(
        "See [apply](applying.md), [boot](./boot-options.md#grub), [below](#steps), [GitLab](https://gitlab.com) and [mail](mailto:x@y.z).\n",
    );

    let spans = paragraph(&blocks[0]);
    let links: Vec<_> = spans.iter().filter_map(|span| span.link.clone()).collect();
    assert_eq!(
        links,
        [
            Link::Page {
                file: "applying.md".into(),
                anchor: None
            },
            Link::Page {
                file: "boot-options.md".into(),
                anchor: Some("grub".into())
            },
            Link::Anchor("steps".into()),
            Link::Web("https://gitlab.com".into()),
        ]
    );
    assert!(
        spans
            .iter()
            .any(|span| span.text == "mail" && span.link.is_none())
    );
}

#[test]
fn an_image_alone_in_its_paragraph_is_an_image_block() {
    let blocks = parse("![The Kernels page](images/kernels.png)\n");

    assert_eq!(
        blocks,
        [Block::Image(Image {
            src: "images/kernels.png".into(),
            alt: "The Kernels page".into(),
        })]
    );
}

#[test]
fn tight_and_loose_lists_hold_their_items_as_blocks() {
    let blocks = parse("1. One **bold**\n2. Two\n\n- Loose\n\n- Items\n");

    let Block::List(numbered) = &blocks[0] else {
        panic!("a list, got {blocks:?}");
    };

    assert_eq!(numbered.start, Some(1));
    assert_eq!(numbered.items.len(), 2);

    assert_eq!(paragraph(&numbered.items[0][0])[0], plain("One "));

    let Block::List(bullets) = &blocks[1] else {
        panic!("a second list, got {blocks:?}");
    };

    assert_eq!(bullets.start, None);
    assert_eq!(bullets.items[1], [Block::Paragraph(vec![plain("Items")])]);
}

#[test]
fn tables_keep_their_head_and_rows() {
    let blocks = parse("| Part | Meaning |\n| --- | --- |\n| `a` | The a |\n| **b** | The b |\n");

    let Block::Table(table) = &blocks[0] else {
        panic!("a table, got {blocks:?}");
    };

    assert_eq!(table.head, [vec![plain("Part")], vec![plain("Meaning")]]);
    assert_eq!(table.rows.len(), 2);

    assert!(table.rows[0][0][0].style.code);
    assert!(table.rows[1][0][0].style.strong);
}

#[test]
fn a_note_or_warning_quote_is_a_callout_without_its_label() {
    let blocks = parse(
        "> **Warning:** A system without a kernel does not boot.\n\n> **Note:** Fine.\n\n> Plain.\n",
    );

    let callouts: Vec<_> = blocks
        .iter()
        .map(|block| {
            let Block::Callout(callout) = block else {
                panic!("a callout, got {block:?}");
            };

            callout
        })
        .collect();

    let kinds: Vec<_> = callouts.iter().map(|callout| callout.kind).collect();
    assert_eq!(
        kinds,
        [CalloutKind::Warning, CalloutKind::Note, CalloutKind::Quote]
    );

    assert_eq!(
        callouts[0].blocks[0],
        Block::Paragraph(vec![plain("A system without a kernel does not boot.")])
    );
}

#[test]
fn code_blocks_keep_their_text_and_language() {
    let blocks = parse("```sh\ngaruda-toolbox status\n```\n\n    indented\n");

    assert_eq!(
        blocks,
        [
            Block::Code(Code {
                language: Some("sh".into()),
                text: "garuda-toolbox status".into(),
            }),
            Block::Code(Code {
                language: None,
                text: "indented".into(),
            }),
        ]
    );
}

#[test]
fn soft_breaks_become_spaces_and_rules_stay() {
    let blocks = parse("one\ntwo\n\n---\n");

    assert_eq!(
        blocks,
        [
            Block::Paragraph(vec![plain("one"), plain(" "), plain("two")]),
            Block::Rule
        ]
    );
}

#[test]
fn raw_html_is_left_out() {
    let blocks = parse("<div>hidden</div>\n\nshown <b>x</b>\n");

    assert_eq!(blocks.len(), 1);
    assert!(
        paragraph(&blocks[0])
            .iter()
            .all(|span| !span.text.contains('<'))
    );
}

#[test]
fn plain_text_joins_every_span_of_a_block() {
    let blocks = parse("- Turn **on** the `switch`\n- Apply\n");

    assert_eq!(blocks[0].plain_text(), "Turn on the switch Apply");
}
