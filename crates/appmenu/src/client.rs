//! A menu client as Plasma's, for the self-test, which clicks the window's
//! menu by an item's label.

#[cfg(test)]
mod tests;

use zbus::{Connection, Proxy, Variant};

use crate::Error;
use crate::server::PATH;
use crate::wire::Item;

/// Clicks the enabled item labelled `label` in the menu `service` serves.
/// `false` when the menu has no such item.
///
/// # Errors
///
/// When the menu cannot be read or refuses the click.
pub async fn click(connection: &Connection, service: &str, label: &str) -> Result<bool, Error> {
    let menu = Proxy::new(connection, service, PATH, "com.canonical.dbusmenu").await?;
    let (_, root): (u32, Item) = menu
        .call("GetLayout", &(0_i32, -1_i32, &["label", "enabled"][..]))
        .await?;

    let Some(id) = find(&root, label)? else {
        return Ok(false);
    };

    let nothing = Variant::from(0_i32);
    menu.call::<_, _, ()>("Event", &(id, "clicked", nothing, 0_u32))
        .await?;
    Ok(true)
}

fn find(item: &Item, label: &str) -> Result<Option<i32>, Error> {
    let shown = item
        .properties
        .get("label")
        .and_then(|value| String::try_from(value.try_clone().ok()?).ok())
        .map(|escaped| escaped.replace("__", "_"));

    let enabled = item
        .properties
        .get("enabled")
        .and_then(|value| bool::try_from(value).ok())
        .unwrap_or(true);

    if enabled && shown.as_deref() == Some(label) {
        return Ok(Some(item.id));
    }

    for child in &item.children {
        if let Some(id) = find(&Item::try_from(child.try_clone()?)?, label)? {
            return Ok(Some(id));
        }
    }

    Ok(None)
}
