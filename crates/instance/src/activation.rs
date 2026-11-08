//! What a start asks of the app: maybe a page, maybe an action of that
//! page, and the tokens that let the window come to the front.
//!
//! Everything here can come from another process over the bus, so each
//! string is checked and bounded before it becomes a value. Unknown
//! platform data keys are ignored, as the specification asks.

#[cfg(test)]
mod tests;

use toolbox_plan::PageName;

const OPEN_PAGE: &str = "open-page";

// The keys of `platform_data` this app reads, after the Desktop Entry
// specification's D-Bus activation section.
const ACTIVATION_TOKEN: &str = "activation-token";
const DESKTOP_STARTUP_ID: &str = "desktop-startup-id";

/// Real tokens are a few dozen bytes. The bound keeps a hostile caller
/// from parking megabytes in the app.
const TOKEN_MAX: usize = 1024;

/// An activation token (Wayland) or a startup id (X11): opaque, visible
/// ASCII without spaces.
#[derive(Clone, PartialEq, Eq)]
pub struct Token(String);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Activation {
    pub page: Option<PageName>,
    /// Only with a page: an action alone would act on whatever page shows.
    pub action: Option<PageName>,
    /// `XDG_ACTIVATION_TOKEN`, for `xdg_activation_v1` on Wayland.
    pub token: Option<Token>,
    /// `DESKTOP_STARTUP_ID`, for the X11 startup notification.
    pub startup_id: Option<Token>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Call<'a> {
    Activate,
    ActivateAction {
        name: &'static str,
        parameters: Vec<&'a str>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    UnknownAction,
    BadParameter,
}

impl Token {
    pub fn new(text: &str) -> Option<Self> {
        let valid = (1..=TOKEN_MAX).contains(&text.len())
            && text.bytes().all(|byte| byte.is_ascii_graphic());

        valid.then(|| Self(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The token lets whoever has it take the focus, so the log shows only
/// that there is one.
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({} bytes)", self.0.len())
    }
}

impl Refusal {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownAction => "Garuda Toolbox has no such action",
            Self::BadParameter => "the action wants a page name and maybe an action name",
        }
    }
}

impl Activation {
    pub fn from_env(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let token = |key: &str| lookup(key).as_deref().and_then(Token::new);

        Self {
            token: token("XDG_ACTIVATION_TOKEN"),
            startup_id: token("DESKTOP_STARTUP_ID"),
            ..Self::default()
        }
    }

    /// # Errors
    ///
    /// An unknown action or parameters that name no page.
    pub fn from_action(name: &str, parameters: &[&str]) -> Result<Self, Refusal> {
        if name != OPEN_PAGE {
            return Err(Refusal::UnknownAction);
        }

        let page_name = |text: &str| PageName::new(text).map_err(|_invalid| Refusal::BadParameter);
        let (page, action) = match parameters {
            [page] => (page_name(page)?, None),
            [page, action] => (page_name(page)?, Some(page_name(action)?)),
            _ => return Err(Refusal::BadParameter),
        };

        Ok(Self {
            page: Some(page),
            action,
            ..Self::default()
        })
    }

    /// Takes the tokens out of `platform_data`. A value that is no string
    /// arrives as `None` and is ignored like an unknown key.
    #[must_use = "returns the new value, the old one stays as it was"]
    pub fn with_platform_data<'a>(
        mut self,
        data: impl IntoIterator<Item = (&'a str, Option<&'a str>)>,
    ) -> Self {
        for (key, value) in data {
            let token = value.and_then(Token::new);
            match key {
                ACTIVATION_TOKEN => self.token = token.or(self.token),
                DESKTOP_STARTUP_ID => self.startup_id = token.or(self.startup_id),
                _ => {}
            }
        }

        self
    }

    pub fn platform_data(&self) -> Vec<(&'static str, &str)> {
        [
            (ACTIVATION_TOKEN, self.token.as_ref()),
            (DESKTOP_STARTUP_ID, self.startup_id.as_ref()),
        ]
        .into_iter()
        .filter_map(|(key, token)| token.map(|token| (key, token.as_str())))
        .collect()
    }

    pub fn call(&self) -> Call<'_> {
        let Some(page) = &self.page else {
            return Call::Activate;
        };

        let mut parameters = vec![page.as_ref()];
        parameters.extend(self.action.as_ref().map(AsRef::as_ref));

        Call::ActivateAction {
            name: OPEN_PAGE,
            parameters,
        }
    }
}
