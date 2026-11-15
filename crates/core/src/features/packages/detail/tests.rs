use toolbox_plan::PackageName;
use toolbox_state::packages::{
    IndexEntry, InstallReason, Installed, LocalNode, Origin, PackageIndex,
};

use super::{Shown, by_kind, file_list, matching_files, shown, target, web_address};
use crate::features::packages::list::Catalog;

fn name(text: &str) -> PackageName {
    PackageName::new(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn entry(text: &str, installed: bool) -> IndexEntry {
    IndexEntry {
        name: name(text),
        description: String::new(),
        version: "1.0-1".to_owned(),
        origin: Origin::Repository("core".to_owned()),
        size: 1,
        installed: installed.then(|| Installed {
            version: "1.0-1".to_owned(),
            reason: InstallReason::Dependency,
        }),
    }
}

fn catalog() -> Catalog {
    Catalog::new(catalog_index())
}

fn catalog_index() -> PackageIndex {
    PackageIndex {
        entries: vec![
            entry("bash", true),
            entry("glibc", true),
            entry("zsh", false),
        ],
        local: vec![
            LocalNode {
                name: name("bash"),
                depends: vec!["glibc".to_owned()],
                provides: vec!["sh=5.2".to_owned()],
                reason: InstallReason::Dependency,
            },
            LocalNode {
                name: name("glibc"),
                depends: Vec::new(),
                provides: Vec::new(),
                reason: InstallReason::Dependency,
            },
        ],
    }
}

#[test]
fn a_dependency_leads_to_the_package_of_its_name_without_the_version() {
    assert_eq!(target(&catalog(), "glibc>=2.40"), Some(name("glibc")));
    assert_eq!(target(&catalog(), "zsh"), Some(name("zsh")));
}

#[test]
fn a_provided_name_leads_to_the_installed_package_that_provides_it() {
    assert_eq!(target(&catalog(), "sh"), Some(name("bash")));
}

#[test]
fn a_name_nothing_offers_or_provides_leads_nowhere() {
    assert_eq!(target(&catalog(), "libfoo.so=1-64"), None);
    assert_eq!(target(&catalog(), ""), None);
}

#[test]
fn a_long_list_shows_its_first_part_and_counts_the_rest() {
    let names: Vec<u8> = (0..15).collect();

    let folded = shown(&names, 12, false);
    assert_eq!(folded.items.len(), 12);
    assert_eq!(folded.hidden, 3);

    let all = shown(&names, 12, true);
    assert_eq!(
        all,
        Shown {
            items: &names[..],
            hidden: 0,
        }
    );
}

#[test]
fn a_short_list_shows_whole() {
    let names = [1_u8, 2];

    assert_eq!(
        shown(&names, 12, false),
        Shown {
            items: &names[..],
            hidden: 0,
        }
    );
}

#[test]
fn the_copied_file_list_has_one_path_per_line() {
    let files = ["/usr/".to_owned(), "/usr/bin/gimp".to_owned()];

    assert_eq!(file_list(&files), "/usr/\n/usr/bin/gimp\n");
    assert_eq!(file_list(&[]), "");
}

#[test]
fn a_web_address_shows_without_its_scheme_and_closing_slash() {
    assert_eq!(
        web_address("https://www.mozilla.org/firefox/").as_deref(),
        Some("www.mozilla.org/firefox")
    );
    assert_eq!(web_address("http://gimp.org").as_deref(), Some("gimp.org"));
}

#[test]
fn an_address_that_is_no_web_link_is_no_link() {
    assert_eq!(web_address("file:///etc/passwd"), None);
    assert_eq!(web_address("--help"), None);
    assert_eq!(web_address("https://"), None);
}

#[test]
fn the_first_installed_package_that_provides_a_name_answers_for_it() {
    let mut index = catalog_index();
    index.local.push(LocalNode {
        name: name("zsh"),
        depends: Vec::new(),
        provides: vec!["sh".to_owned()],
        reason: InstallReason::Explicit,
    });

    assert_eq!(target(&Catalog::new(index), "sh"), Some(name("bash")));
}

/// mpv's own dependencies, as its pacman database entry lists them.
#[test]
fn package_names_come_first_and_the_sonames_of_libraries_after() {
    let depends = [
        "ffmpeg",
        "libasound.so=2-64",
        "glibc",
        "libavcodec.so=63-64",
        "libGL.so",
    ]
    .map(str::to_owned);

    let split = by_kind(&depends);

    assert_eq!(split.packages, ["ffmpeg", "glibc"]);
    assert_eq!(
        split.libraries,
        ["libasound.so=2-64", "libavcodec.so=63-64", "libGL.so"]
    );
}

#[test]
fn a_package_named_like_a_library_is_a_package() {
    let depends = ["libsoup3", "libso.sos"].map(str::to_owned);

    assert_eq!(by_kind(&depends).libraries, Vec::<&str>::new());
}

#[test]
fn the_file_filter_matches_any_part_of_a_path_in_either_case() {
    let files = ["/usr/", "/usr/bin/gimp", "/usr/share/GIMP/"].map(str::to_owned);

    assert_eq!(matching_files(&files, ""), [0, 1, 2]);
    assert_eq!(matching_files(&files, "gimp"), [1, 2]);
    assert_eq!(matching_files(&files, " bin/G "), [1]);
    assert_eq!(matching_files(&files, "kate"), Vec::<usize>::new());
}
