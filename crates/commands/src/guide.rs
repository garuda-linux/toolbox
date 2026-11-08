#[cfg(test)]
mod tests;

pub mod terminal;

use std::io::Write;

use serde::Serialize;
use toolbox_cli::Exit;
use toolbox_cli::args::{GuideCommand, GuideShow, ReadList};
use toolbox_cli::output::{Paint, Row, Style, json};
use toolbox_guide::Page;

use crate::filter::{self, Named};
use crate::out::{self, Listing};

/// The width text wraps at when the output is no terminal.
const PIPE_WIDTH: usize = 80;

#[derive(Clone, Debug, Serialize)]
struct GuideRow {
    /// The page's name for `guide show`: its file without `.md`.
    #[serde(rename = "id")]
    page: String,
    #[serde(rename = "name")]
    title: String,
    part: Option<String>,
}

#[derive(Serialize)]
struct Shown<'a> {
    #[serde(rename = "id")]
    page: &'a str,
    #[serde(rename = "name")]
    title: &'a str,
    part: Option<&'a str>,
    outline: Vec<Heading<'a>>,
    text: String,
}

#[derive(Serialize)]
struct Heading<'a> {
    level: u8,
    anchor: &'a str,
    text: &'a str,
}

impl Named for GuideRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.page, &self.title]
    }
}

pub fn command(command: GuideCommand, out: &mut dyn Write) -> Exit {
    match command {
        GuideCommand::List(list) => self::list(&list, out),
        GuideCommand::Show(args) => show(&args, out),
    }
}

fn list(list: &ReadList, out: &mut dyn Write) -> Exit {
    let rows: Vec<GuideRow> = toolbox_guide::embedded()
        .pages()
        .iter()
        .map(|page| GuideRow {
            page: name_of(page).to_owned(),
            title: page.title.clone(),
            part: page.part.clone(),
        })
        .collect();

    filter::print_all(list, &rows, out, |rows| Listing {
        title: "Guide",
        summary: Some(rows.len().to_string()),
        head: &["PAGE", "TITLE", "PART"],
        rows: rows
            .iter()
            .map(|row| {
                Row::plain(vec![
                    row.page.clone(),
                    row.title.clone(),
                    row.part.clone().unwrap_or_default(),
                ])
            })
            .collect(),
        hint: Some("garuda-toolbox guide show <page>".to_owned()),
    })
}

fn show(args: &GuideShow, out: &mut dyn Write) -> Exit {
    let file = format!("{}.md", args.page.as_ref());
    let Some(page) = toolbox_guide::embedded().page(&file) else {
        let what = format!(
            "no page `{}` in the guide, see `guide list`",
            args.page.as_ref()
        );

        return out::unavailable(&what, out);
    };

    let paint = out::paint();
    if args.read.json {
        let shown = Shown {
            page: name_of(page),
            title: &page.title,
            part: page.part.as_deref(),
            outline: page
                .outline
                .iter()
                .map(|heading| Heading {
                    level: heading.level,
                    anchor: &heading.anchor,
                    text: &heading.text,
                })
                .collect(),
            text: terminal::page(page, Paint::new(Style::Plain), PIPE_WIDTH),
        };

        write!(out, "{}", json(&shown)).ok();
        return Exit::Success;
    }

    let width = out::columns().unwrap_or(PIPE_WIDTH);
    write!(out, "{}", terminal::page(page, paint, width)).ok();
    Exit::Success
}

/// The page's name on the command line: its file without `.md`.
pub fn name_of(page: &Page) -> &str {
    page.file.strip_suffix(".md").unwrap_or(&page.file)
}
