use super::Named;
use crate::boot::ModuleRow;
use crate::configs::FileRow;
use crate::lists::{GroupRow, KernelRow, LocaleRow, ServiceRow, ToggleRow, UserRow};
use crate::locales::PackRow;
use crate::network::DriverLine;
use crate::settings::SettingRow;

impl Named for ServiceRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.name]
    }
}

impl Named for ToggleRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.id, &self.name]
    }
}

impl Named for KernelRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.name]
    }
}

impl Named for LocaleRow {
    fn names(&self) -> Vec<&str> {
        std::iter::once(self.locale.as_str())
            .chain(self.name.as_deref())
            .collect()
    }
}

impl Named for UserRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.name, &self.full_name]
    }
}

impl Named for GroupRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.name]
    }
}

impl Named for ModuleRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.module]
    }
}

impl Named for FileRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.key, &self.name]
    }
}

impl Named for PackRow {
    fn names(&self) -> Vec<&str> {
        vec![&self.package, &self.program]
    }
}

impl Named for DriverLine {
    fn names(&self) -> Vec<&str> {
        vec![&self.module]
    }
}

impl Named for SettingRow {
    fn names(&self) -> Vec<&str> {
        vec![self.key]
    }
}
