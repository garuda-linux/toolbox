mod wiki;

use toolbox_catalog::{Check, ToggleCatalog};
use toolbox_plan::PackageName;

use super::tasks::{finish, package_diff, parse_options, shipped_packages};
use super::writer::{entry_id, quoted, render};
use super::{App, CATEGORIES, Lists, https_url, wiki_text};

fn app(title: &str, packages: &[&str]) -> App {
    App {
        title: title.to_owned(),
        description: format!("About {title}."),
        url: None,
        packages: packages.iter().map(|name| (*name).to_owned()).collect(),
    }
}

#[test]
fn ids_come_from_the_first_package() {
    assert_eq!(
        entry_id(&app("Firefox", &["firefox"])),
        Some("firefox".to_owned())
    );
    assert_eq!(
        entry_id(&app("Write", &["write_stylus"])),
        Some("write-stylus".to_owned())
    );
    assert_eq!(
        entry_id(&app("Nicotine+", &["nicotine+"])),
        Some("nicotine-plus".to_owned())
    );
    assert_eq!(
        entry_id(&app("Speedtest++", &["speedtest++"])),
        Some("speedtest-plus-plus".to_owned())
    );
    assert_eq!(
        entry_id(&app("Gimp", &["GIMP.bin"])),
        Some("gimp-bin".to_owned())
    );
    assert_eq!(entry_id(&app("None", &[])), None);
    assert_eq!(entry_id(&app(" ", &["blank"])), None);
}

#[test]
fn toml_strings_escape_quotes_backslashes_and_controls() {
    assert_eq!(quoted("plain"), "\"plain\"");
    assert_eq!(quoted("say \"hi\""), "\"say \\\"hi\\\"\"");
    assert_eq!(quoted("C:\\dos"), "\"C:\\\\dos\"");
    assert_eq!(quoted("a\nb\tc\u{7}"), "\"a\\nb\\tc\\u0007\"");
}

#[test]
fn the_written_catalog_is_a_valid_toggle_catalog() {
    let category = CATEGORIES[2];
    let apps = [
        App {
            title: "Fire\"fox\\".to_owned(),
            description: "Web \"browser\".".to_owned(),
            url: Some("https://www.mozilla.org/firefox/".to_owned()),
            packages: vec!["firefox".to_owned(), "firefox-i18n-de".to_owned()],
        },
        app("Nicotine+", &["nicotine+"]),
    ]
    .map(|app| (entry_id(&app).unwrap(), app));

    let text = render(category, &apps);
    let catalog = ToggleCatalog::parse(&text, "internet.toml").unwrap();
    let section = &catalog.sections()[0];

    assert_eq!(section.id(), "internet");
    assert_eq!(section.title(), "Internet");
    assert_eq!(section.icon(), "applications-internet");

    let firefox = catalog.entry("firefox").unwrap();

    assert_eq!(firefox.title(), "Fire\"fox\\");
    assert_eq!(firefox.description(), "Web \"browser\".");
    assert_eq!(firefox.url(), Some("https://www.mozilla.org/firefox/"));
    assert_eq!(
        firefox.check(),
        &Check::Package(PackageName::new("firefox").unwrap())
    );
    assert_eq!(
        firefox.also(),
        [PackageName::new("firefox-i18n-de").unwrap()]
    );
    assert_eq!(catalog.entry("nicotine-plus").unwrap().url(), None);
    assert_eq!(
        render(category, &apps),
        text,
        "the same input writes the same file"
    );
}

#[test]
fn a_package_listed_twice_keeps_its_first_entry() {
    let lists = Lists::new(vec![
        (
            CATEGORIES[0],
            vec![app("A", &["a"]), app("A again", &["a"]), app("B", &["b"])],
        ),
        (
            CATEGORIES[1],
            vec![app("B game", &["b"]), app("C", &["c"]), app("", &["d"])],
        ),
    ]);

    let dir = tempfile::tempdir().unwrap();
    let summary = lists.write(dir.path()).unwrap();

    assert_eq!(
        lists.dropped(),
        [
            "documents: a repeats",
            "games: b repeats",
            "games: \"\" is incomplete"
        ]
    );
    assert!(summary.contains("documents: 2 apps"), "{summary}");
    assert!(summary.contains("games: 1 apps"), "{summary}");

    let games = std::fs::read_to_string(dir.path().join("games.toml")).unwrap();

    assert_eq!(
        ToggleCatalog::parse(&games, "games.toml")
            .unwrap()
            .sections()[0]
            .entries()
            .len(),
        1
    );
}

#[test]
fn wiki_markup_becomes_plain_text() {
    let cases = [
        ("'''Bold''' and ''italic''", "Bold and italic"),
        (
            "See [[Firefox|the browser]] or [[Chromium]].",
            "See the browser or Chromium.",
        ),
        ("[[Wikipedia:Tetris|Tetris]] clone", "Tetris clone"),
        ("Uses {{Pkg|sdl2}} and {{AUR|foo|bar}}", "Uses sdl2 and foo"),
        ("{{Grp|kde-games}}", "Part of KDE Games"),
        (
            "Run {{ic|foo --bar}} {{Dead link|2023|05|01}}",
            "Run foo --bar",
        ),
        ("See {{man|1|ls}} now", "See ls now"),
        ("a{{=}}b &amp; c", "a=b & c"),
        (
            "Home [https://example.org the site] here",
            "Home the site here",
        ),
        (
            "Generates man pages.[https://archlinux.org/pacman/pacman.8.html].",
            "Generates man pages.",
        ),
        ("Based on the <map> tag", "Based on the <map> tag"),
        ("  many \t blanks  ", "many blanks"),
        ("A mesh VPN network ", "A mesh VPN network"),
        ("polished GTK GUI .", "polished GTK GUI."),
        ("Downloader for .nzb files", "Downloader for .nzb files"),
    ];

    for (wiki, plain) in cases {
        assert_eq!(wiki_text(wiki), plain, "{wiki}");
    }

    assert_eq!(wiki_text(&wiki_text(cases[1].0)), cases[1].1, "idempotent");
}

#[test]
fn only_the_first_https_link_is_kept() {
    assert_eq!(
        https_url("https://github.com/derf/feh https://feh.finalrewind.org"),
        Some("https://github.com/derf/feh".to_owned())
    );
    assert_eq!(https_url("http://acme.cat-v.org/"), None);
    assert_eq!(https_url(""), None);
    assert_eq!(https_url("https://user@host.org"), None);
}

#[test]
fn every_category_has_its_wiki_page_and_file() {
    let pages: Vec<String> = CATEGORIES
        .iter()
        .map(|category| category.wiki_page())
        .collect();

    assert_eq!(pages[1], "List_of_games");
    assert_eq!(pages[0], "List_of_applications/Documents");
    assert_eq!(pages[7], "List_of_applications/Utilities");
    assert_eq!(CATEGORIES[6].file_name(), "security.toml");

    let files: Vec<String> = CATEGORIES
        .iter()
        .map(|category| format!("apps/{}", category.file_name()))
        .collect();

    assert_eq!(
        files,
        toolbox_catalog::APPS,
        "the app reads what xtask writes"
    );
}

#[test]
fn package_diffs_name_what_comes_and_goes() {
    use std::collections::BTreeSet;

    let old: BTreeSet<String> = ["a", "b"].into_iter().map(str::to_owned).collect();
    let new: BTreeSet<String> = ["b", "c"].into_iter().map(str::to_owned).collect();
    let lines = package_diff(&old, &new);

    assert!(lines.contains(&"package added: c".to_owned()), "{lines:?}");
    assert!(
        lines.contains(&"package removed: a".to_owned()),
        "{lines:?}"
    );
    assert_eq!(package_diff(&old, &old), Vec::<String>::new());
}

#[test]
fn shipped_packages_lists_checks_and_companions() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::write(
        dir.path().join("internet.toml"),
        "[[section]]\nid = \"internet\"\ntitle = \"Internet\"\nicon = \"applications-internet\"\n\n\
         [[section.entry]]\nid = \"firefox\"\ntitle = \"Firefox\"\ndescription = \"Browser.\"\n\
         check = { package = \"firefox\" }\nalso = [{ package = \"firefox-i18n-de\" }]\n",
    )
    .unwrap();

    let packages = shipped_packages(dir.path()).unwrap();

    assert!(packages.contains("firefox"));
    assert!(packages.contains("firefox-i18n-de"));
    assert_eq!(packages.len(), 2);
}

/// A wiki page that broke would empty its catalog: the run shows the
/// difference first and writes only when asked.
#[test]
fn a_sync_writes_nothing_without_write() {
    let folder = tempfile::tempdir().unwrap();
    let lists = Lists::new(vec![(CATEGORIES[2], vec![app("Firefox", &["firefox"])])]);

    let report = finish(&lists, folder.path(), false).unwrap();

    assert!(report.contains("--write"), "{report}");
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);

    finish(&lists, folder.path(), true).unwrap();
    assert!(std::fs::read_dir(folder.path()).unwrap().count() > 0);
}

#[test]
fn write_is_an_option_of_its_own() {
    let options = parse_options(&["--write".to_owned()]).unwrap();

    assert!(options.write);
    assert!(!parse_options(&[]).unwrap().write);
}

fn pages(dir: &std::path::Path) {
    for category in CATEGORIES {
        std::fs::write(
            dir.join(format!("{}.txt", category.id)),
            "* {{App|Firefox|Extensible browser.|https://www.firefox.com/|{{Pkg|firefox}}}}\n",
        )
        .unwrap();
    }
}

fn args(args: &[&str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_owned()).collect()
}

#[test]
fn a_sync_over_saved_pages_reports_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let local = dir.path().join("pages");
    let out = dir.path().join("out");
    std::fs::create_dir_all(&local).unwrap();
    pages(&local);

    let report = match super::tasks::wiki_sync(&args(&[
        "--local",
        local.to_str().unwrap(),
        "--unfiltered",
        "--out",
        out.to_str().unwrap(),
    ])) {
        Ok(report) => report,
        Err(crate::Failure::Usage) => panic!("saved pages fail the arguments"),
        Err(crate::Failure::Problems(lines)) => panic!("saved pages fail: {lines:?}"),
    };

    assert!(
        report.contains("documents: 1 apps found, 0 not offered"),
        "{report}"
    );
    assert!(report.contains("dry run: nothing written"), "{report}");
    assert!(!out.exists(), "a dry run writes no catalog");
}

#[test]
fn missing_pages_fail_the_sync() {
    let dir = tempfile::tempdir().unwrap();

    match super::tasks::wiki_sync(&args(&[
        "--local",
        dir.path().to_str().unwrap(),
        "--unfiltered",
    ])) {
        Err(crate::Failure::Problems(_)) => {}
        Err(crate::Failure::Usage) => panic!("missing pages fail the arguments"),
        Ok(report) => panic!("missing pages reported a sync:\n{report}"),
    }
}

#[test]
fn a_sources_flag_needs_its_folder() {
    assert!(parse_options(&args(&["--sources"])).is_none());
}

/// A broken catalog would count as empty and the diff would call every
/// package new, so the run stops on it instead.
#[test]
fn a_broken_shipped_catalog_stops_the_diff() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::write(dir.path().join("internet.toml"), "[[section\n").unwrap();

    let error = shipped_packages(dir.path()).unwrap_err();

    assert!(error.contains("internet.toml"), "{error}");
}

#[test]
fn an_unreadable_shipped_catalog_stops_the_diff() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::create_dir_all(dir.path().join("internet.toml")).unwrap();

    let error = shipped_packages(dir.path()).unwrap_err();

    assert!(error.contains("internet.toml"), "{error}");
}

#[test]
fn missing_catalogs_and_non_package_checks_add_no_package() {
    let dir = tempfile::tempdir().unwrap();

    std::fs::write(
        dir.path().join("documents.toml"),
        "[[section]]\nid = \"documents\"\ntitle = \"Documents\"\nicon = \"x\"\n\n\
         [[section.entry]]\nid = \"cups\"\ntitle = \"Printing\"\ndescription = \"P.\"\n\
         check = { service = \"cups.service\" }\n",
    )
    .unwrap();

    assert!(shipped_packages(dir.path()).unwrap().is_empty());
}

/// su is a command of util-linux: its switch would remove the package
/// that logs everyone in. sudo is its own package and stays listed.
#[test]
fn a_command_of_a_protected_package_is_left_out() {
    let lists = Lists::new(vec![(
        CATEGORIES[6],
        vec![app("su", &["util-linux"]), app("sudo", &["sudo"])],
    )])
    .without_commands_of();

    assert_eq!(lists.packages().into_iter().collect::<Vec<_>>(), ["sudo"]);
    assert_eq!(
        lists.dropped(),
        ["security: \"su\" is a command of the protected util-linux"]
    );
}
