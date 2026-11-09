//! Each part is a single word dkms takes as an argument, so a build can
//! name nothing but a module below `/var/lib/dkms` and a kernel below
//! `/usr/lib/modules`.

use serde::{Deserialize, Serialize};

use crate::procedure::{Command, Program};
use crate::{DkmsModuleName, DkmsVersion, KernelRelease};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DkmsBuild {
    pub module: DkmsModuleName,
    pub version: DkmsVersion,
    pub kernel: KernelRelease,
}

impl DkmsBuild {
    /// Without `--no-depmod`, as nothing runs depmod afterwards and the
    /// kernel would not find the module it just installed.
    #[must_use = "a command does nothing until it is run"]
    pub fn command(&self) -> Command {
        Command::new(Program::Dkms)
            .arg("install")
            .arg(format!("{}/{}", self.module, self.version))
            .args(["-k", self.kernel.as_str()])
    }
}
