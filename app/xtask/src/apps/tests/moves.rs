//! On the shipped Documents catalog.

use super::super::moves::{firsts, gate, moves};
use super::super::tasks::shipped;
use super::super::{CATEGORIES, Lists, TARGET_DIR};
use super::app;
use crate::root;

fn shipped_firsts() -> super::super::moves::Firsts {
    firsts(&shipped(&root().join(TARGET_DIR)).unwrap())
}

fn documents(apps: Vec<super::super::App>) -> Lists {
    Lists::new(vec![(CATEGORIES[0], apps)])
}

#[test]
fn an_app_the_wiki_lists_with_another_first_package_is_a_move() {
    // Changed: the wiki's AbiWord names abiword-gtk2 first.
    let lists = documents(vec![
        app("AbiWord", &["abiword-gtk2"]),
        app("Acme", &["plan9port"]),
    ]);

    assert_eq!(
        moves(&shipped_firsts(), &lists),
        ["first package moved: documents: \"AbiWord\" from abiword to abiword-gtk2"]
    );
}

#[test]
fn a_new_app_and_an_unchanged_one_are_no_move() {
    let lists = documents(vec![
        app("AbiWord", &["abiword"]),
        app("Not shipped yet", &["not-shipped"]),
    ]);

    assert_eq!(moves(&shipped_firsts(), &lists), Vec::<String>::new());
}

#[test]
fn a_write_with_moves_needs_their_acceptance() {
    let found = ["first package moved: documents: \"AbiWord\"".to_owned()];

    assert_eq!(
        gate(&found, false, false),
        Ok(()),
        "a dry run only lists them"
    );
    assert_eq!(gate(&found, true, true), Ok(()));
    assert_eq!(gate(&[], true, false), Ok(()));

    let refused = gate(&found, true, false).unwrap_err();

    assert_eq!(refused.len(), 2);
    assert!(refused[1].contains("--accept-moves"));
}
