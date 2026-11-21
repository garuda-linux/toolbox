use std::sync::Arc;

use futures_lite::StreamExt as _;
use parking_lot::Mutex;
use zbus::{OwnedVariant, Proxy, Str, Variant};

use super::{Exported, PATH};
use crate::menu::{Entry, Item, Menu, Modifier, Shortcut, Submenu, Toggle};
use crate::testing::pair;
use crate::wire;

type Clicks = Arc<Mutex<Vec<&'static str>>>;

/// `ItemsPropertiesUpdated`: the changed and the removed properties.
type PropertyNews = (Vec<(i32, wire::Properties)>, Vec<(i32, Vec<String>)>);

/// A menu client as Plasma's, over a peer connection.
struct Client(Proxy<'static>);

impl Client {
    async fn layout(
        &self,
        parent: i32,
        depth: i32,
        names: &[&str],
    ) -> zbus::Result<(u32, wire::Item)> {
        self.0.call("GetLayout", &(parent, depth, names)).await
    }

    async fn click(&self, id: i32, event: &str) -> zbus::Result<()> {
        let nothing = Variant::from(0_i32);
        self.0.call("Event", &(id, event, nothing, 0_u32)).await
    }

    async fn about_to_show(&self, id: i32) -> zbus::Result<bool> {
        self.0.call("AboutToShow", &(id,)).await
    }

    async fn property(&self, id: i32, name: &str) -> zbus::Result<OwnedVariant> {
        self.0.call("GetProperty", &(id, name)).await
    }
}

fn menu(zoom_enabled: bool) -> Menu<&'static str> {
    let mut zoom = Item::new("zoom", "Zoom_in".into());
    zoom.enabled = zoom_enabled;
    zoom.shortcut = Some(Shortcut {
        modifiers: vec![Modifier::Control],
        key: "plus",
    });

    let mut sidebar = Item::new("sidebar", "Sidebar".into());
    sidebar.toggle = Toggle::Check(true);

    Menu {
        menus: vec![Submenu {
            label: "View".into(),
            entries: vec![Entry::Item(sidebar), Entry::Separator, Entry::Item(zoom)],
        }],
    }
}

async fn served(menu: &Menu<&'static str>) -> (Exported<&'static str>, Client, Clicks) {
    let (server, client) = pair(|builder| builder).await;
    let clicks = Clicks::default();
    let kept = Arc::clone(&clicks);
    let exported = Exported::serve(server, menu, move |action| kept.lock().push(action))
        .await
        .unwrap();

    // A peer connection ignores the destination, but zbus wants one.
    let proxy = Proxy::new(
        &client,
        "org.garudalinux.toolbox.Test",
        PATH,
        "com.canonical.dbusmenu",
    )
    .await
    .unwrap();

    (exported, Client(proxy), clicks)
}

fn label(item: &wire::Item) -> Option<String> {
    String::try_from(item.properties.get("label")?.try_clone().ok()?).ok()
}

fn child(item: &wire::Item, index: usize) -> wire::Item {
    wire::Item::try_from(item.children[index].try_clone().unwrap()).unwrap()
}

fn text(value: &'static str) -> OwnedVariant {
    OwnedVariant::from(Str::from_static(value))
}

#[tokio::test]
async fn the_layout_lists_each_menu_with_its_entries_and_their_properties() {
    let (_exported, client, _) = served(&menu(true)).await;

    let (revision, root) = client.layout(0, -1, &[]).await.unwrap();

    assert_eq!(revision, 1);
    assert_eq!(client.0.get_property::<u32>("Version").await.unwrap(), 3);
    let view = child(&root, 0);
    assert_eq!(label(&view).as_deref(), Some("View"));
    assert_eq!(view.children.len(), 3);
    let zoom = child(&view, 2);
    assert_eq!(
        label(&zoom).as_deref(),
        Some("Zoom__in"),
        "an underscore is no access key"
    );
    let shortcut: Vec<Vec<String>> = zoom.properties["shortcut"]
        .try_clone()
        .unwrap()
        .try_into()
        .unwrap();

    assert_eq!(shortcut, [["Control", "plus"]]);
    let sidebar = child(&view, 0);
    assert_eq!(sidebar.properties["toggle-type"], text("checkmark"));
    assert_eq!(
        sidebar.properties["toggle-state"],
        OwnedVariant::from(1_i32)
    );
    assert_eq!(child(&view, 1).properties["type"], text("separator"));
}

#[tokio::test]
async fn a_depth_and_property_names_trim_the_layout() {
    let (_exported, client, _) = served(&menu(true)).await;

    let (_, root) = client.layout(0, 1, &["label"]).await.unwrap();

    let view = child(&root, 0);
    assert_eq!(view.children.len(), 0);
    assert_eq!(view.properties.keys().collect::<Vec<_>>(), ["label"]);
}

#[tokio::test]
async fn a_click_hands_the_action_over_but_not_for_a_disabled_item_or_a_separator() {
    let (_exported, client, clicks) = served(&menu(false)).await;

    client.click(2, "clicked").await.unwrap();
    client.click(2, "hovered").await.unwrap();
    client.click(3, "clicked").await.unwrap();
    client.click(4, "clicked").await.unwrap();

    assert_eq!(*clicks.lock(), ["sidebar"]);
}

#[tokio::test]
async fn an_unknown_item_is_refused() {
    let (_exported, client, clicks) = served(&menu(true)).await;

    client.click(99, "clicked").await.unwrap_err();
    client.about_to_show(-1).await.unwrap_err();
    client.layout(99, -1, &[]).await.unwrap_err();
    assert!(clicks.lock().is_empty());
}

#[tokio::test]
async fn a_changed_state_sends_only_the_changed_item() {
    let (exported, client, _) = served(&menu(true)).await;
    let mut stream = client
        .0
        .receive_signal("ItemsPropertiesUpdated")
        .await
        .unwrap();

    exported.update(&menu(false)).await.unwrap();

    let message = stream.next().await.unwrap();
    let (updated, removed): PropertyNews = message.body().deserialize().unwrap();
    let ids: Vec<i32> = updated.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, [4_i32]);
    assert_eq!(removed.len(), 0);
    assert_eq!(updated[0].1["enabled"], OwnedVariant::from(false));
    assert_eq!(
        client.property(4, "enabled").await.unwrap(),
        OwnedVariant::from(false)
    );
}

#[tokio::test]
async fn a_new_shape_raises_the_revision_and_asks_for_a_new_read() {
    let (exported, client, _) = served(&menu(true)).await;
    let mut layouts = client.0.receive_signal("LayoutUpdated").await.unwrap();
    let mut longer = menu(true);
    longer.menus[0]
        .entries
        .push(Entry::Item(Item::new("reset", "Reset".into())));

    exported.update(&longer).await.unwrap();

    let message = layouts.next().await.unwrap();
    let news: (u32, i32) = message.body().deserialize().unwrap();
    assert_eq!(news, (2_u32, 0_i32));
    let (revision, root) = client.layout(0, -1, &[]).await.unwrap();
    assert_eq!(revision, 2);
    assert_eq!(child(&root, 0).children.len(), 4);
}
