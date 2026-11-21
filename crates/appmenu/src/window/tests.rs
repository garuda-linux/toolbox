use std::sync::Arc;

use parking_lot::Mutex;
use zbus::{Connection, OwnedObjectPath, fdo};

use super::{REGISTRAR_PATH, register};
use crate::testing::pair;

type Registered = Arc<Mutex<Vec<(u32, String)>>>;

/// Plasma's Registrar, or one that is not running.
struct Registrar {
    registered: Registered,
    running: bool,
}

#[zbus::interface(name = "com.canonical.AppMenu.Registrar")]
impl Registrar {
    #[expect(
        clippy::needless_pass_by_value,
        reason = "zbus hands the arguments of a method over owned"
    )]
    fn register_window(
        &self,
        window_id: u32,
        menu_object_path: OwnedObjectPath,
    ) -> fdo::Result<()> {
        if !self.running {
            return Err(fdo::Error::ServiceUnknown("no registrar".into()));
        }

        self.registered
            .lock()
            .push((window_id, menu_object_path.to_string()));
        Ok(())
    }
}

async fn client_of(running: bool) -> (Connection, Connection, Registered) {
    let registered = Registered::default();
    let registrar = Registrar {
        registered: Arc::clone(&registered),
        running,
    };

    let (server, client) = pair(|builder| builder.serve_at(REGISTRAR_PATH, registrar)).await;
    (server, client, registered)
}

#[tokio::test]
async fn an_x11_window_registers_its_id_with_the_menu_path() {
    let (_server, client, registered) = client_of(true).await;

    assert!(register(&client, 0x0420_0007).await.unwrap());
    assert_eq!(*registered.lock(), [(0x0420_0007, "/MenuBar".to_owned())]);
}

#[tokio::test]
async fn without_a_registrar_the_window_has_no_global_menu() {
    let (_server, client, registered) = client_of(false).await;

    assert!(!register(&client, 7).await.unwrap());
    assert_eq!(registered.lock().len(), 0);
}
