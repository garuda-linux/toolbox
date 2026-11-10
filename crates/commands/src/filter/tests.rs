use toolbox_cli::args::{ReadList, Search, ServicesList, ToggleList};
use toolbox_cli::output::{Mark, Row};

use super::{Filter, Named, kept, narrow, print, services, summary, toggles};
use crate::lists::{LocaleRow, ServiceRow, ToggleRow};
use crate::out::{Listing, Out, Settings};

const FAKES: [Fake; 3] = [
    Fake {
        id: "docker",
        name: "Docker",
        on: true,
    },
    Fake {
        id: "podman",
        name: "Podman containers",
        on: false,
    },
    Fake {
        id: "steam",
        name: "Steam",
        on: true,
    },
];

#[derive(Clone, serde::Serialize)]
struct Fake {
    id: &'static str,
    name: &'static str,
    on: bool,
}

impl Named for Fake {
    fn names(&self) -> Vec<&str> {
        vec![self.id, self.name]
    }
}

fn ids(rows: &[&Fake]) -> Vec<&'static str> {
    rows.iter().map(|row| row.id).collect()
}

fn fake_listing(rows: &[Fake]) -> Listing<'_> {
    Listing {
        title: "Fakes",
        summary: Some(format!(
            "{}, {} on",
            rows.len(),
            rows.iter().filter(|row| row.on).count()
        )),
        head: &[],
        rows: rows
            .iter()
            .map(|row| Row::new(Mark::On, vec![row.id.to_owned()]))
            .collect(),
        hint: Some(format!("{} left", rows.len())),
    }
}

#[test]
fn without_a_filter_nothing_is_narrowed() {
    assert!(kept(&FAKES, None, &Filter::all()).is_none());
    assert!(
        kept(
            &FAKES,
            None,
            &Filter::all().only(false, |row: &Fake| row.on)
        )
        .is_none()
    );
}

#[test]
fn a_search_matches_the_id_and_the_shown_name() {
    let search = Search::new("CONTAINER");
    let found = kept(&FAKES, Some(&search), &Filter::all()).unwrap();

    assert_eq!(ids(&found), ["podman"]);
}

#[test]
fn filters_and_the_search_all_apply() {
    let search = Search::new("e");
    let filter = Filter::all().only(true, |row: &Fake| row.on);
    let found = kept(&FAKES, Some(&search), &filter).unwrap();

    assert_eq!(ids(&found), ["docker", "steam"]);

    let narrower = kept(&FAKES, Some(&Search::new("st")), &filter).unwrap();
    assert_eq!(ids(&narrower), ["steam"]);
}

#[test]
fn the_summary_counts_the_whole_list_and_the_shown_rows() {
    assert_eq!(
        summary(Some("266, 40 running".to_owned()), 12),
        "266, 40 running, 12 shown"
    );
    assert_eq!(summary(None, 0), "0 shown");
}

#[test]
fn a_narrowed_listing_keeps_the_whole_summary_and_the_shown_rows() {
    let shown = [FAKES[2].clone()];
    let listing = narrow(fake_listing(&FAKES), fake_listing(&shown), shown.len());

    assert_eq!(listing.title, "Fakes");
    assert_eq!(listing.summary.as_deref(), Some("3, 2 on, 1 shown"));
    assert_eq!(listing.rows.len(), 1);
    assert_eq!(listing.hint.as_deref(), Some("1 left"));
}

#[test]
fn json_holds_only_the_kept_rows() {
    let list = ReadList {
        json: true,
        search: Some(Search::new("steam")),
    };

    let mut out = Vec::new();
    print(
        &list,
        &FAKES,
        &Filter::all(),
        &mut Out::new(&mut out, Settings::default()),
        fake_listing,
    );

    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("steam") && !text.contains("docker"), "{text}");
}

#[test]
fn text_says_how_many_are_shown() {
    let list = ReadList {
        json: false,
        search: Some(Search::new("o")),
    };

    let mut out = Vec::new();
    print(
        &list,
        &FAKES,
        &Filter::all(),
        &mut Out::new(&mut out, Settings::default()),
        fake_listing,
    );

    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("3, 2 on, 2 shown"), "{text}");
    assert!(text.contains("podman") && !text.contains("steam"), "{text}");
}

fn service(name: &str, active: &'static str, at_boot: bool) -> ServiceRow {
    ServiceRow {
        name: name.to_owned(),
        active,
        state: String::new(),
        at_boot,
        description: String::new(),
    }
}

fn services_list(running: bool, failed: bool, enabled: bool) -> ServicesList {
    ServicesList {
        user: false,
        running,
        failed,
        enabled,
        read: ReadList::default(),
    }
}

#[test]
fn services_filter_by_state_and_boot() {
    let rows = [
        service("a.service", "active", true),
        service("b.service", "active", false),
        service("c.service", "failed", true),
        service("d.service", "inactive", true),
    ];

    let names = |list: ServicesList| -> Vec<String> {
        kept(&rows, None, &services(&list))
            .map(|rows| rows.into_iter().map(|row| row.name.clone()).collect())
            .unwrap_or_default()
    };

    assert_eq!(
        names(services_list(true, false, false)),
        ["a.service", "b.service"]
    );
    assert_eq!(names(services_list(false, true, false)), ["c.service"]);
    assert_eq!(names(services_list(true, false, true)), ["a.service"]);
    assert_eq!(names(services_list(false, false, true)).len(), 3);
}

fn toggle(id: &str, state: &'static str) -> ToggleRow {
    ToggleRow {
        page: "gaming",
        section: String::new(),
        id: id.to_owned(),
        name: id.to_uppercase(),
        state,
        available: true,
    }
}

#[test]
fn switches_filter_by_on_and_off_and_unknown_is_neither() {
    let rows = [
        toggle("a", "on"),
        toggle("b", "off"),
        toggle("c", "unknown"),
    ];

    let list = |on, off| ToggleList {
        page: None,
        on,
        off,
        read: ReadList::default(),
    };

    let on = kept(&rows, None, &toggles(&list(true, false))).unwrap();
    assert_eq!(on.len(), 1);
    assert_eq!(on[0].id, "a");

    let off = kept(&rows, None, &toggles(&list(false, true))).unwrap();
    assert_eq!(off.len(), 1);
    assert_eq!(off[0].id, "b");
}

#[test]
fn a_locale_matches_its_language_name() {
    let row = LocaleRow {
        locale: "de_DE.UTF-8".to_owned(),
        charset: "UTF-8".to_owned(),
        name: Some("German (Germany)".to_owned()),
        enabled: false,
        system: false,
    };

    assert!(Search::new("german").matches(row.names()));
    assert!(Search::new("de_de").matches(row.names()));
}
