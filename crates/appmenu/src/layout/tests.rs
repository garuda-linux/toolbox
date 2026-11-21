use super::{Layout, ROOT};
use crate::menu::{Entry, Item, Menu, Submenu, Toggle};

fn item(action: &'static str) -> Entry<&'static str> {
    Entry::Item(Item::new(action, action.to_uppercase()))
}

fn menu(file: Vec<Entry<&'static str>>, view: Vec<Entry<&'static str>>) -> Menu<&'static str> {
    Menu {
        menus: vec![
            Submenu {
                label: "File".into(),
                entries: file,
            },
            Submenu {
                label: "View".into(),
                entries: view,
            },
        ],
    }
}

fn plain() -> Menu<&'static str> {
    menu(vec![item("open"), Entry::Separator], vec![item("zoom")])
}

#[test]
fn the_root_holds_the_menus_and_each_menu_its_entries_in_order() {
    let layout = Layout::of(&plain()).unwrap();

    assert_eq!(layout.node(ROOT).unwrap().children, [1_i32, 4_i32]);
    assert_eq!(layout.node(1).unwrap().children, [2_i32, 3_i32]);
    assert_eq!(layout.node(4).unwrap().children, [5_i32]);
    assert_eq!(layout.node(1).unwrap().props.label.as_deref(), Some("File"));
    assert_eq!(layout.node(2).unwrap().action, Some("open"));
    assert!(layout.node(3).unwrap().props.separator);
    assert!(layout.node(1).unwrap().props.submenu);
    assert!(layout.node(6).is_none());
}

#[test]
fn a_state_change_keeps_the_shape_and_names_only_the_changed_item() {
    let old = Layout::of(&plain()).unwrap();
    let mut disabled = plain();
    if let Some(Entry::Item(zoom)) = disabled.menus[1].entries.first_mut() {
        zoom.enabled = false;
    }

    let new = Layout::of(&disabled).unwrap();

    assert!(new.same_shape(&old));
    let changed: Vec<i32> = new.changed(&old).into_iter().map(|(id, _)| id).collect();
    assert_eq!(changed, [5_i32]);
}

#[test]
fn a_new_entry_or_a_new_check_mark_changes_the_shape() {
    let old = Layout::of(&plain()).unwrap();
    let longer = menu(
        vec![item("open"), Entry::Separator],
        vec![item("zoom"), item("reset")],
    );

    let mut checked = plain();
    if let Some(Entry::Item(zoom)) = checked.menus[1].entries.first_mut() {
        zoom.toggle = Toggle::Check(true);
    }

    assert!(!Layout::of(&longer).unwrap().same_shape(&old));
    assert!(!Layout::of(&checked).unwrap().same_shape(&old));
}
