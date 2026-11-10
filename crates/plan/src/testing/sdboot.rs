pub const GARUDA_7_3_1_1_GARUDA_EFI_HEAD: &[u8] =
    include_bytes!("../sdboot/fixtures/garuda-7.3.1-1-garuda.efi.head");
pub const GARUDA_7_3_1_1_GARUDA_FALLBACK_EFI_HEAD: &[u8] =
    include_bytes!("../sdboot/fixtures/garuda-7.3.1-1-garuda-fallback.efi.head");
pub const LOADER_CONF: &str = include_str!("../sdboot/fixtures/loader.conf");
pub const MOUNTINFO: &str = include_str!("../sdboot/fixtures/mountinfo");
