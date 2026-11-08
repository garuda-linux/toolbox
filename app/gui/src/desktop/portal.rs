use ashpd::desktop::settings::{APPEARANCE_NAMESPACE, ColorScheme, Settings};
use futures_lite::{Stream, StreamExt as _};
use toolbox_gui_kit::desktop::look::Portal;

/// GNOME's interface settings, which the portal passes on where the
/// desktop keeps them in gsettings.
const INTERFACE_NAMESPACE: &str = "org.gnome.desktop.interface";
const ENABLE_ANIMATIONS: &str = "enable-animations";

/// The portal's settings proxy. `None` without a portal, as on a bare
/// window manager, which then keeps the files and the fallback.
pub async fn connect() -> Option<Settings> {
    match Settings::new().await {
        Ok(settings) => Some(settings),
        Err(error) => {
            tracing::info!("no desktop portal, the look comes from files alone: {error}");
            None
        }
    }
}

/// What the portal says now. A setting the portal lacks counts as unset.
pub async fn read(settings: Option<&Settings>) -> Portal {
    let Some(settings) = settings else {
        return Portal::default();
    };

    let dark = match settings.color_scheme().await {
        Ok(ColorScheme::PreferDark) => Some(true),
        Ok(ColorScheme::PreferLight) => Some(false),
        Ok(ColorScheme::NoPreference) | Err(_) => None,
    };

    let accent =
        settings.accent_color().await.ok().and_then(|color| {
            Portal::accent_from_unit_rgb(color.red(), color.green(), color.blue())
        });

    let animations = settings
        .read::<bool>(INTERFACE_NAMESPACE, ENABLE_ANIMATIONS)
        .await
        .ok();

    Portal {
        dark,
        accent,
        animations,
    }
}

/// One item per change of an appearance or a GNOME interface setting,
/// as the second holds the animations. Empty without a portal
/// or when the portal refuses the subscription.
pub async fn changes<'settings>(
    settings: Option<&'settings Settings>,
) -> Box<dyn Stream<Item = ()> + Send + Unpin + 'settings> {
    let Some(settings) = settings else {
        return Box::new(futures_lite::stream::empty());
    };

    match settings.receive_setting_changed().await {
        Ok(stream) => Box::new(Box::pin(
            stream
                .filter(|setting| {
                    [APPEARANCE_NAMESPACE, INTERFACE_NAMESPACE].contains(&setting.namespace())
                })
                .map(|_| ()),
        )),
        Err(error) => {
            tracing::info!("the desktop portal does not report changes: {error}");
            Box::new(futures_lite::stream::empty())
        }
    }
}
