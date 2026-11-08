use toolbox_core::Toggle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscoverCommand {
    Toggle { app: String, wish: Toggle },
    Reveal(String),
}
