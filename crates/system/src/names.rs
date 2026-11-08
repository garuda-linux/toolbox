//! Turning probe text into value objects.

use std::collections::BTreeSet;

use toolbox_plan::{Name, Rule};

/// The valid names among `values`. A system can hold names our rules
/// reject (a hand-made package, an odd unit). Those cannot be changed
/// through the app anyway, so they are logged and left out.
pub fn valid<'text, R, I>(values: I) -> BTreeSet<Name<R>>
where
    R: Rule,
    I: IntoIterator<Item = &'text str>,
{
    values.into_iter().filter_map(one).collect()
}

/// `value` as a name, or `None` (logged) when our rules reject it.
pub fn one<R>(value: &str) -> Option<Name<R>>
where
    R: Rule,
{
    Name::new(value)
        .inspect_err(|error| tracing::debug!("skipped: {error}"))
        .ok()
}
