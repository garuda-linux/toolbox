//! The menu in dbusmenu's types: an item is `(ia{sv}av)`, its children
//! items again inside variants.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use zbus::{OwnedVariant, Str, Type, Variant};

use crate::layout::{Layout, Node, Props};

pub type Properties = HashMap<String, OwnedVariant>;

#[derive(Debug, Deserialize, Serialize, Type, Variant, OwnedVariant)]
pub struct Item {
    pub id: i32,
    pub properties: Properties,
    pub children: Vec<OwnedVariant>,
}

/// `node` with its children down to `depth` levels, all of them when
/// `depth` is negative.
pub fn item<A>(
    layout: &Layout<A>,
    (id, node): (i32, &Node<A>),
    depth: i32,
    names: &[String],
) -> zbus::Result<Item> {
    let mut children = Vec::new();
    if depth != 0_i32 {
        let below = if depth < 0_i32 { depth } else { depth - 1_i32 };
        for &child in &node.children {
            if let Some(found) = layout.node(child) {
                let built = item(layout, (child, found), below, names)?;
                children.push(OwnedVariant::try_from(Variant::from(built))?);
            }
        }
    }

    Ok(Item {
        id,
        properties: properties(&node.props, names)?,
        children,
    })
}

/// The properties `names` asks for, all of them when it is empty.
pub fn properties(props: &Props, names: &[String]) -> zbus::Result<Properties> {
    let mut all: Vec<(&str, OwnedVariant)> =
        vec![("visible", true.into()), ("enabled", props.enabled.into())];

    if let Some(label) = &props.label {
        all.push(("label", Str::from(escaped(label)).into()));
    }

    if props.separator {
        all.push(("type", Str::from_static("separator").into()));
    }

    if props.submenu {
        all.push(("children-display", Str::from_static("submenu").into()));
    }

    if let (Some(kind), Some(state)) = (props.toggle.kind(), props.toggle.state()) {
        all.push(("toggle-type", Str::from_static(kind).into()));
        all.push(("toggle-state", state.into()));
    }

    if let Some(shortcut) = &props.shortcut {
        let chords = Variant::from(shortcut.chords());
        all.push(("shortcut", OwnedVariant::try_from(chords)?));
    }

    Ok(all
        .into_iter()
        .filter(|(name, _)| names.is_empty() || names.iter().any(|asked| asked == name))
        .map(|(name, value)| (name.to_owned(), value))
        .collect())
}

/// dbusmenu reads `_` as the mark of an access key, `__` as the character.
fn escaped(label: &str) -> String {
    label.replace('_', "__")
}
