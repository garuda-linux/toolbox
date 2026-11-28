//! The desktop portal's appearance settings and GNOME's interface settings:
//! the dark preference, the accent colour and whether animations run. Without
//! a portal every setting is unset and no change comes.

use std::sync::Arc;

use ashpd::desktop::settings::{APPEARANCE_NAMESPACE, ColorScheme, Settings};
use futures_lite::StreamExt as _;
use futures_lite::future::Boxed;
use tokio::sync::OnceCell;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// GNOME's interface settings, which the portal passes on where the desktop
/// keeps them in gsettings.
const INTERFACE_NAMESPACE: &str = "org.gnome.desktop.interface";
const ENABLE_ANIMATIONS: &str = "enable-animations";

/// What the portal says about the look. `None` is no preference, or a
/// setting the portal lacks.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PortalSettings {
    /// `Some(true)` for a dark preference.
    pub dark: Option<bool>,
    /// Red, green and blue from 0 to 1.
    pub accent: Option<(f64, f64, f64)>,
    /// GNOME's `enable-animations`.
    pub animations: Option<bool>,
}

pub trait LookPortal: Send + Sync {
    /// What the portal says now.
    fn read(&self) -> Boxed<PortalSettings>;

    /// One item per change of an appearance or GNOME interface setting. It
    /// is closed at once without a portal or when the portal refuses the
    /// subscription.
    fn changes(&self) -> Boxed<UnboundedReceiver<()>>;
}

/// The session's portal, connected at the first use and kept for as long as
/// the app runs.
#[derive(Clone, Debug, Default)]
pub struct SystemLookPortal {
    settings: Arc<OnceCell<Option<Settings>>>,
}

impl SystemLookPortal {
    async fn settings(cell: &OnceCell<Option<Settings>>) -> Option<&Settings> {
        cell.get_or_init(connect).await.as_ref()
    }
}

impl LookPortal for SystemLookPortal {
    fn read(&self) -> Boxed<PortalSettings> {
        let cell = Arc::clone(&self.settings);
        Box::pin(async move { read_settings(Self::settings(&cell).await).await })
    }

    fn changes(&self) -> Boxed<UnboundedReceiver<()>> {
        let cell = Arc::clone(&self.settings);
        Box::pin(async move {
            let (sender, receiver) = unbounded_channel();
            let connected = Self::settings(&cell).await.is_some();
            if connected {
                tokio::spawn(forward(cell, sender));
            }

            receiver
        })
    }
}

async fn connect() -> Option<Settings> {
    match Settings::new().await {
        Ok(settings) => Some(settings),
        Err(error) => {
            tracing::info!("no desktop portal, the look comes from files alone: {error}");
            None
        }
    }
}

async fn read_settings(settings: Option<&Settings>) -> PortalSettings {
    let Some(settings) = settings else {
        return PortalSettings::default();
    };

    let dark = match settings.color_scheme().await {
        Ok(ColorScheme::PreferDark) => Some(true),
        Ok(ColorScheme::PreferLight) => Some(false),
        Ok(ColorScheme::NoPreference) | Err(_) => None,
    };

    let accent = settings
        .accent_color()
        .await
        .ok()
        .map(|color| (color.red(), color.green(), color.blue()));

    let animations = settings
        .read::<bool>(INTERFACE_NAMESPACE, ENABLE_ANIMATIONS)
        .await
        .ok();

    PortalSettings {
        dark,
        accent,
        animations,
    }
}

/// Sends a unit for each change until the receiver is dropped or the portal
/// ends its signals.
async fn forward(cell: Arc<OnceCell<Option<Settings>>>, sender: UnboundedSender<()>) {
    let Some(Some(settings)) = cell.get() else {
        return;
    };

    let stream = match settings.receive_setting_changed().await {
        Ok(stream) => stream,
        Err(error) => {
            tracing::info!("the desktop portal does not report changes: {error}");
            return;
        }
    };

    let mut changes = Box::pin(
        stream
            .filter(|setting| {
                [APPEARANCE_NAMESPACE, INTERFACE_NAMESPACE].contains(&setting.namespace())
            })
            .map(|_| ()),
    );

    while changes.next().await.is_some() {
        if sender.send(()).is_err() {
            return;
        }
    }
}
