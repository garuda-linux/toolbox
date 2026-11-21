//! `com.canonical.dbusmenu`, as libdbusmenu and Plasma's menu read it.
//!
//! Any process of the user can call it. An event only hands over the
//! action of an enabled item it names, and the app decides what to do.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use parking_lot::Mutex;
use zbus::object_server::SignalEmitter;
use zbus::{Connection, OwnedVariant, fdo};

use crate::Error;
use crate::layout::{Layout, Node, ROOT};
use crate::menu::Menu;
use crate::wire::{self, Item, Properties};

/// Where the menu is served, as Qt serves its own.
pub const PATH: &str = "/MenuBar";

/// A menu served on a connection. Dropping it leaves the menu served
/// until the connection closes.
pub struct Exported<A> {
    connection: Connection,
    state: Arc<Mutex<State<A>>>,
}

struct State<A> {
    /// Raised with every new shape, so a client knows to read it again.
    revision: u32,
    layout: Layout<A>,
}

struct DbusMenu<A> {
    state: Arc<Mutex<State<A>>>,
    clicks: Box<dyn Fn(A) + Send + Sync>,
}

impl<A: Clone + Send + Sync + 'static> Exported<A> {
    /// Serves `menu` at [`PATH`] on `connection`. `clicks` runs on the
    /// bus's threads.
    ///
    /// # Errors
    ///
    /// When the menu is too large to number or the path is taken.
    pub async fn serve(
        connection: Connection,
        menu: &Menu<A>,
        clicks: impl Fn(A) + Send + Sync + 'static,
    ) -> Result<Self, Error> {
        let state = Arc::new(Mutex::new(State {
            revision: 1,
            layout: Layout::of(menu)?,
        }));

        let server = DbusMenu {
            state: Arc::clone(&state),
            clicks: Box::new(clicks),
        };

        connection.object_server().at(PATH, server).await?;
        Ok(Self { connection, state })
    }

    /// Shows `menu` from now on. A client that already read the menu
    /// hears only what changed, or to read it again when its shape did.
    ///
    /// # Errors
    ///
    /// When the menu is too large to number or a signal cannot be sent.
    pub async fn update(&self, menu: &Menu<A>) -> Result<(), Error> {
        let new = Layout::of(menu)?;
        let news = {
            let mut state = self.state.lock();
            let news = if new.same_shape(&state.layout) {
                let updated = new
                    .changed(&state.layout)
                    .into_iter()
                    .map(|(id, props)| Ok((id, wire::properties(props, &[])?)))
                    .collect::<zbus::Result<Vec<_>>>()?;

                News::Properties(updated)
            } else {
                state.revision = state.revision.wrapping_add(1);
                News::Layout(state.revision)
            };

            state.layout = new;
            news
        };

        let emitter = SignalEmitter::new(&self.connection, PATH)?;
        match news {
            News::Properties(updated) if updated.is_empty() => {}
            News::Properties(updated) => {
                DbusMenu::<A>::items_properties_updated(&emitter, updated, Vec::new()).await?;
            }
            News::Layout(revision) => {
                DbusMenu::<A>::layout_updated(&emitter, revision, ROOT).await?;
            }
        }

        Ok(())
    }

    pub const fn connection(&self) -> &Connection {
        &self.connection
    }
}

enum News {
    Properties(Vec<(i32, Properties)>),
    Layout(u32),
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl<A: Clone + Send + Sync + 'static> DbusMenu<A> {
    #[zbus(signal)]
    async fn items_properties_updated(
        emitter: &SignalEmitter<'_>,
        updated_props: Vec<(i32, Properties)>,
        removed_props: Vec<(i32, Vec<String>)>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn layout_updated(
        emitter: &SignalEmitter<'_>,
        revision: u32,
        parent: i32,
    ) -> zbus::Result<()>;

    #[expect(
        clippy::needless_pass_by_value,
        reason = "zbus hands the arguments of a method over owned"
    )]
    fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: Vec<String>,
    ) -> fdo::Result<(u32, Item)> {
        let state = self.state.lock();
        let node = known(&state.layout, parent_id)?;
        let item = wire::item(
            &state.layout,
            (parent_id, node),
            recursion_depth,
            &property_names,
        )?;

        Ok((state.revision, item))
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "zbus hands the arguments of a method over owned"
    )]
    fn get_group_properties(
        &self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> fdo::Result<Vec<(i32, Properties)>> {
        let state = self.state.lock();
        let asked: Vec<i32> = if ids.is_empty() {
            state.layout.nodes().map(|(id, _)| id).collect()
        } else {
            ids
        };

        let mut found = Vec::new();
        for id in asked {
            if let Some(node) = state.layout.node(id) {
                found.push((id, wire::properties(&node.props, &property_names)?));
            }
        }

        Ok(found)
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "zbus hands the arguments of a method over owned"
    )]
    fn get_property(&self, id: i32, name: String) -> fdo::Result<OwnedVariant> {
        let props = known(&self.state.lock().layout, id)?.props.clone();
        wire::properties(&props, std::slice::from_ref(&name))?
            .remove(&name)
            .ok_or_else(|| fdo::Error::InvalidArgs(format!("item {id} has no property {name}")))
    }

    #[expect(
        clippy::needless_pass_by_value,
        unused_variables,
        reason = "the specification's arguments, which zbus hands over owned and the menu needs no data or time of"
    )]
    fn event(
        &self,
        id: i32,
        event_id: String,
        data: OwnedVariant,
        timestamp: u32,
    ) -> fdo::Result<()> {
        let action = {
            let state = self.state.lock();
            clicked(known(&state.layout, id)?, &event_id)
        };

        if let Some(action) = action {
            (self.clicks)(action);
        }

        Ok(())
    }

    /// The ids it did not know. An error when it knew none of them.
    #[expect(
        clippy::needless_pass_by_value,
        reason = "zbus hands the arguments of a method over owned"
    )]
    fn event_group(&self, events: Vec<(i32, String, OwnedVariant, u32)>) -> fdo::Result<Vec<i32>> {
        let mut missing = Vec::new();
        let mut actions = Vec::new();
        {
            let state = self.state.lock();
            for (id, event_id, _, _) in &events {
                match state.layout.node(*id) {
                    Some(node) => actions.extend(clicked(node, event_id)),
                    None => missing.push(*id),
                }
            }
        }

        if !events.is_empty() && missing.len() == events.len() {
            return Err(fdo::Error::InvalidArgs(
                "no item of the group exists".into(),
            ));
        }

        for action in actions {
            (self.clicks)(action);
        }

        Ok(missing)
    }

    /// The menu is always up to date, so nothing needs a new layout.
    fn about_to_show(&self, id: i32) -> fdo::Result<bool> {
        known(&self.state.lock().layout, id).map(|_| false)
    }

    fn about_to_show_group(&self, ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        let state = self.state.lock();
        let missing = ids
            .into_iter()
            .filter(|id| state.layout.node(*id).is_none())
            .collect();

        (Vec::new(), missing)
    }

    #[expect(
        clippy::unused_self,
        reason = "the specification's property, constant here"
    )]
    #[zbus(property)]
    const fn version(&self) -> u32 {
        3
    }

    #[expect(
        clippy::unused_self,
        reason = "the specification's property, constant here"
    )]
    #[zbus(property)]
    fn text_direction(&self) -> String {
        "ltr".into()
    }

    #[expect(
        clippy::unused_self,
        reason = "the specification's property, constant here"
    )]
    #[zbus(property)]
    fn status(&self) -> String {
        "normal".into()
    }

    #[expect(
        clippy::unused_self,
        reason = "the specification's property, constant here"
    )]
    #[zbus(property)]
    const fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }
}

fn known<A>(layout: &Layout<A>, id: i32) -> fdo::Result<&Node<A>> {
    layout
        .node(id)
        .ok_or_else(|| fdo::Error::InvalidArgs(format!("no item {id}")))
}

/// The action a click on `node` runs: none for a menu, a separator or a
/// disabled item.
fn clicked<A>(node: &Node<A>, event_id: &str) -> Option<A>
where
    A: Clone,
{
    if event_id == "clicked" && node.props.enabled {
        node.action.clone()
    } else {
        None
    }
}
