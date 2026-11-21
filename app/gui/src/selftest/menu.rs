//! The global menu over the session bus, as Plasma clicks it. The app
//! names its menu's bus address once it serves it.

use std::thread;

use crossbeam_channel::Receiver;
use egui::{Context, Id};

/// Whether an enabled item took the click, or why the click failed.
pub type Answer = Receiver<Result<bool, String>>;

pub fn announce(ctx: &Context, service: String) {
    ctx.data_mut(|data| data.insert_temp(id(), service));
}

/// Clicks the item labelled `label` on a thread of its own. `None` while
/// the app serves no menu yet.
pub fn click(ctx: &Context, label: &'static str) -> Option<Result<Answer, String>> {
    let service: String = ctx.data(|data| data.get_temp(id()))?;
    let (sender, answer) = crossbeam_channel::bounded(1);
    let spawned = thread::Builder::new()
        .name("selftest-menu".into())
        .spawn(move || {
            let clicked = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())
                .and_then(|runtime| runtime.block_on(clicked(&service, label)));

            drop(sender.send(clicked));
        });

    Some(spawned.map(|_| answer).map_err(|error| error.to_string()))
}

async fn clicked(service: &str, label: &str) -> Result<bool, String> {
    let connection = zbus::Connection::session()
        .await
        .map_err(|error| error.to_string())?;

    toolbox_appmenu::client::click(&connection, service, label)
        .await
        .map_err(|error| error.to_string())
}

fn id() -> Id {
    Id::new("toolbox_gui::selftest::menu")
}
