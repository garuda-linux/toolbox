//! Package names written into the source. A `const` item runs the check
//! while the crate compiles, so a typo fails the build and the running app
//! has no error path for a name that cannot be wrong.

#[cfg(test)]
mod tests;

use std::marker::PhantomData;

use super::Name;
use super::rules::{MAX_PACKAGE_CHARS, PackageName};

#[derive(Clone, Copy, Debug)]
pub struct KnownPackage(&'static str);

impl KnownPackage {
    /// Meant for `const` items, where a bad name stops the build.
    ///
    /// # Panics
    ///
    /// When `value` is no package name and the call runs outside a `const`
    /// item.
    #[must_use = "a known package is only useful as its name"]
    pub const fn new(value: &'static str) -> Self {
        assert!(is_package_name(value), "not a package name");
        Self(value)
    }

    pub fn name(self) -> PackageName {
        Name {
            value: self.0.to_owned(),
            rule: PhantomData,
        }
    }
}

/// [`super::rules::PackageRule`] in a form `const` code can run. Both
/// accept only ASCII, so bytes count as characters.
const fn is_package_name(value: &str) -> bool {
    let bytes = value.as_bytes();
    let [first, ..] = bytes else {
        return false;
    };

    if bytes.len() > MAX_PACKAGE_CHARS
        || !(first.is_ascii_alphanumeric() || matches!(*first, b'@' | b'_' | b'+'))
    {
        return false;
    }

    let mut rest = bytes;
    while let [byte, tail @ ..] = rest {
        if !(byte.is_ascii_alphanumeric() || matches!(*byte, b'@' | b'.' | b'_' | b'+' | b'-')) {
            return false;
        }

        rest = tail;
    }

    true
}
