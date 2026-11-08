use toolbox_cli::Exit;
use toolbox_cli::args::{GuideShow, Read, ReadList, Search};
use toolbox_cli::output::{Paint, Style};
use toolbox_guide::Guide;
use toolbox_plan::PageName;

use super::{list, show, terminal};

const SUMMARY: &str = "# Summary\n\n# System\n\n- [Kernels](kernels.md)\n";

const KERNELS: &str = "# Kernels\n\nLists the kernels your repositories offer and the ones you have.\n\n## Install a kernel\n\n1. Turn on the switch.\n2. Apply the queue.\n\n> **Warning:** A system without a kernel does not boot.\n\n| Part | Meaning |\n| --- | --- |\n| `Running` | The kernel the system booted with |\n\n```sh\ngaruda-toolbox kernels list\n```\n\nSee [GitLab](https://gitlab.com/garuda-linux).\n";

fn plain() -> Paint {
    Paint::new(Style::Plain)
}

fn page() -> String {
    let guide = Guide::new(SUMMARY, &[("kernels.md", KERNELS)]);
    let kernels = guide.page("kernels.md").expect("listed");
    terminal::page(kernels, plain(), 40)
}

fn run(command: impl FnOnce(&mut Vec<u8>) -> Exit) -> (Exit, String) {
    let mut out = Vec::new();
    let exit = command(&mut out);
    let text = String::from_utf8(out).unwrap_or_default();
    (exit, text.replace(": ", ":"))
}

#[test]
fn the_title_carries_the_part_and_text_wraps_at_the_width() {
    let text = page();
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines[0], "Kernels  (System)");
    assert!(
        lines.iter().all(|line| line.chars().count() <= 40),
        "{text}"
    );
    assert!(text.contains("Lists the kernels your repositories"));
}

#[test]
fn numbered_steps_keep_their_numbers() {
    let text = page();

    assert!(text.contains("  1. Turn on the switch."), "{text}");
    assert!(text.contains("  2. Apply the queue."), "{text}");
}

#[test]
fn a_warning_is_labelled_and_a_table_keeps_its_rows() {
    let text = page();

    assert!(
        text.contains("Warning: A system without a kernel"),
        "{text}"
    );
    assert!(text.contains("Running"), "{text}");
    assert!(text.contains("The kernel the system"), "{text}");
}

#[test]
fn code_is_indented_and_web_links_show_their_address() {
    let text = page();

    assert!(text.contains("    garuda-toolbox kernels list"), "{text}");
    assert!(text.contains("<https://gitlab.com/garuda-linux>"), "{text}");
}

#[test]
fn list_names_every_page_of_the_built_in_guide() {
    let (exit, text) = run(|out| list(&ReadList::default(), out));

    assert_eq!(exit, Exit::Success);
    assert!(text.contains("kernels"), "{text}");
    assert!(text.contains("Troubleshooting"), "{text}");
}

#[test]
fn list_with_a_search_keeps_the_matching_pages() {
    let search = ReadList {
        json: true,
        search: Some(Search::new("kernel")),
    };

    let (_, text) = run(|out| list(&search, out));

    assert!(text.contains("\"id\":\"kernels\""), "{text}");
    assert!(!text.contains("troubleshooting"), "{text}");
}

#[test]
fn show_prints_a_page_and_json_holds_its_outline() {
    let args = GuideShow {
        page: PageName::new("kernels").expect("a name"),
        read: Read { json: true },
    };

    let (exit, text) = run(|out| show(&args, out));

    assert_eq!(exit, Exit::Success);
    assert!(text.contains("\"name\":\"Kernels\""), "{text}");
    assert!(text.contains("\"anchor\":\"install-a-kernel\""), "{text}");
}

#[test]
fn show_of_an_unknown_page_says_so() {
    let args = GuideShow {
        page: PageName::new("nowhere").expect("a name"),
        read: Read { json: false },
    };

    let (exit, text) = run(|out| show(&args, out));

    assert_eq!(exit, Exit::Unavailable);
    assert_eq!(text, "");
}
