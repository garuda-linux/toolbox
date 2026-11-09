//! Where a file lies in the repository. The rules on programs hold for
//! shipped code only, since test fakes start shells and bare names on
//! purpose.

use std::ffi::OsStr;
use std::path::Path;

pub struct Place<'p> {
    path: &'p Path,
}

impl<'p> Place<'p> {
    pub const fn new(path: &'p Path) -> Self {
        Self { path }
    }

    /// A `tests` or `benches` folder, or a `tests.rs` file.
    pub fn is_test(&self) -> bool {
        self.has_component("tests")
            || self.has_component("benches")
            || self.path.file_name() == Some(OsStr::new("tests.rs"))
    }

    pub fn in_src(&self) -> bool {
        self.has_component("src")
    }

    /// Whether this is `file`, given from the repository root, also when
    /// the linter got an absolute path.
    pub fn is(&self, file: &str) -> bool {
        self.path.ends_with(file)
    }

    fn has_component(&self, name: &str) -> bool {
        self.path
            .components()
            .any(|component| component.as_os_str() == name)
    }
}
