//! The menu as dbusmenu numbers it: the root is 0, then each menu and its
//! entries in order.

#[cfg(test)]
mod tests;

use crate::Error;
use crate::menu::{Entry, Item, Menu, Shortcut, Toggle};

pub const ROOT: i32 = 0;

pub struct Layout<A> {
    nodes: Vec<Node<A>>,
}

pub struct Node<A> {
    pub action: Option<A>,
    pub children: Vec<i32>,
    pub props: Props,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Props {
    pub label: Option<String>,
    pub enabled: bool,
    pub separator: bool,
    pub submenu: bool,
    pub toggle: Toggle,
    pub shortcut: Option<Shortcut>,
}

/// What a client keeps between two layouts: the numbering, the kinds and
/// which properties exist at all.
#[derive(PartialEq, Eq)]
struct Shape<'node> {
    children: &'node [i32],
    separator: bool,
    submenu: bool,
    toggle: Option<&'static str>,
    shortcut: bool,
}

impl<A: Clone> Layout<A> {
    pub fn of(menu: &Menu<A>) -> Result<Self, Error> {
        let mut layout = Self {
            nodes: vec![Node::parent(None)],
        };

        for submenu in &menu.menus {
            let id = layout.push(ROOT, Node::parent(Some(submenu.label.clone())))?;
            for entry in &submenu.entries {
                let node = match entry {
                    Entry::Item(item) => Node::item(item),
                    Entry::Separator => Node::separator(),
                };

                layout.push(id, node)?;
            }
        }

        Ok(layout)
    }
}

impl<A> Layout<A> {
    pub fn node(&self, id: i32) -> Option<&Node<A>> {
        self.nodes.get(usize::try_from(id).ok()?)
    }

    pub fn nodes(&self) -> impl Iterator<Item = (i32, &Node<A>)> {
        (0_i32..).zip(&self.nodes)
    }

    /// Whether a client that read `old` only needs the changed properties.
    pub fn same_shape(&self, old: &Self) -> bool {
        self.nodes.len() == old.nodes.len()
            && self
                .nodes
                .iter()
                .zip(&old.nodes)
                .all(|(new, old)| new.shape() == old.shape())
    }

    pub fn changed(&self, old: &Self) -> Vec<(i32, &Props)> {
        self.nodes()
            .zip(&old.nodes)
            .filter(|((_, new), old)| new.props != old.props)
            .map(|((id, new), _)| (id, &new.props))
            .collect()
    }

    fn push(&mut self, parent: i32, node: Node<A>) -> Result<i32, Error> {
        let id = i32::try_from(self.nodes.len())?;
        self.nodes.push(node);
        if let Some(parent) = usize::try_from(parent)
            .ok()
            .and_then(|index| self.nodes.get_mut(index))
        {
            parent.children.push(id);
        }

        Ok(id)
    }
}

impl<A> Node<A> {
    const fn parent(label: Option<String>) -> Self {
        Self::with(None, Props::new(label, true, false, true))
    }

    const fn separator() -> Self {
        Self::with(None, Props::new(None, true, true, false))
    }

    const fn with(action: Option<A>, props: Props) -> Self {
        Self {
            action,
            children: Vec::new(),
            props,
        }
    }

    fn shape(&self) -> Shape<'_> {
        Shape {
            children: &self.children,
            separator: self.props.separator,
            submenu: self.props.submenu,
            toggle: self.props.toggle.kind(),
            shortcut: self.props.shortcut.is_some(),
        }
    }
}

impl<A: Clone> Node<A> {
    fn item(item: &Item<A>) -> Self {
        let mut props = Props::new(Some(item.label.clone()), item.enabled, false, false);
        props.toggle = item.toggle;
        props.shortcut.clone_from(&item.shortcut);
        Self::with(Some(item.action.clone()), props)
    }
}

impl Props {
    const fn new(label: Option<String>, enabled: bool, separator: bool, submenu: bool) -> Self {
        Self {
            label,
            enabled,
            separator,
            submenu,
            toggle: Toggle::None,
            shortcut: None,
        }
    }
}
