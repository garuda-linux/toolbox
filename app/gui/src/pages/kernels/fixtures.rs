//! The kernels system of `toolbox_core::testing`, with dkms as the page
//! reads it.

use std::sync::Arc;

use toolbox_core::testing::features::kernels;
pub use toolbox_core::testing::features::kernels::{package, system};
use toolbox_services::dkms::Dkms;

use crate::kernels::DkmsReading;

/// `nvidia` is built for `linux`, `vboxhost` for no kernel.
pub fn dkms() -> DkmsReading {
    DkmsReading::Loaded(Arc::new(Dkms::Modules(kernels::dkms())))
}
