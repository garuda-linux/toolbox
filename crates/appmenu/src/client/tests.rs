use std::sync::Arc;

use parking_lot::Mutex;

use super::click;
use crate::menu::{Entry, Item, Menu, Submenu};
use crate::server::Exported;
use crate::testing::pair;

fn menu() -> Menu<&'static str> {
    let mut reset = Item::new("reset", "Reset_zoom".into());
    reset.enabled = false;

    Menu {
        menus: vec![Submenu {
            label: "View".into(),
            entries: vec![
                Entry::Item(Item::new("zoom", "Zoom in".into())),
                Entry::Separator,
                Entry::Item(reset),
            ],
        }],
    }
}

#[tokio::test]
async fn a_click_by_label_reaches_the_item_but_not_a_disabled_or_missing_one() {
    let (server, client) = pair(|builder| builder).await;
    let clicks = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&clicks);
    let _exported = Exported::serve(server, &menu(), move |action| kept.lock().push(action))
        .await
        .unwrap();

    // A peer connection ignores the destination, but zbus wants one.
    let service = "org.garudalinux.toolbox.Test";
    assert!(click(&client, service, "Zoom in").await.unwrap());
    assert!(!click(&client, service, "Reset_zoom").await.unwrap());
    assert!(!click(&client, service, "Quit").await.unwrap());
    assert_eq!(*clicks.lock(), ["zoom"]);
}
