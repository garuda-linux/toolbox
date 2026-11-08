use toolbox_plan::PackageName;
use toolbox_state::packages::{
    IndexEntry, InstallReason, Installed, LocalNode, Origin, PackageIndex,
};

use super::{Shown, file_list, shown, target, web_address};
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
    Catalog::new(PackageIndex {
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
    })
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
