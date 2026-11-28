//! What the app may ask the system to do, and the wire format to the root
//! helper. Everything here is plain data with validation and no I/O, so the
//! helper can link it without pulling in the rest of the app.

#[macro_use]
mod validated;

mod account;
mod action;
mod aur;
mod bounded;
mod component;
mod config_file;
mod dkms;
mod editor;
mod garuda_update;
mod grub;
pub mod lines;
mod locale;
mod merge;
mod mirrorlist;
mod name;
mod network;
mod page_name;
mod plan;
pub mod procedure;
mod protected;
pub mod protocol;
mod repair;
mod scheme;
pub mod sdboot;
mod secret;
mod settings;
mod shell;
mod shown;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
mod toggle;
mod user_config;

pub use account::{
    AccountLock, Caller, FullName, HomeChange, HomeDirectory, HomeFolder, InvalidFullName,
    InvalidHomeDirectory, InvalidNewUser, InvalidUserId, LAST_USER_ID, NewHome, NewUser,
    NewUserParts, PEOPLE, PasswdEntry, SHADOW, ShadowLocks, UserId, passwd_entries,
};
pub use action::{
    EmptyTargetList, LoaderRefusal, RemoteAction, RootAction, TargetList, UnitOperation, UnitScope,
    Upgrader, UserAction, boot_loader_steps,
};
pub use aur::{AUR_BUILT, AUR_CLONE, AUR_STORE, InstallReason, MAKEPKG_ARGS, REVIEW_TAIL};
pub use bounded::{Bounded, OutOfRange};
pub use config_file::{
    CONFIG_ROOTS, ConfigEntryLocation, ConfigFilePath, ConfigText, ConfigWrite, FetchAnswer,
    FetchRefusal, Fingerprint, HOME_CONFIG_BACKUPS, HomeWrite, InvalidConfigFilePath,
    InvalidConfigText, MAX_CONFIG_BYTES, NotWritable, WRITABLE_CONFIG_FILES,
    WRITABLE_CONFIG_FOLDERS,
};
pub use dkms::DkmsBuild;
pub use editor::{EDIT_FOLDER, EditCopy, Editor, InvalidEditCopy};
pub use garuda_update::{GARUDA_UPDATE_CONFIG, GarudaUpdateFlag};
pub use grub::{
    DefaultEntry, EFI_FIRMWARE, GRUB_CONFIG, GRUB_DEFAULTS, GRUB_DEFAULTS_BACKUP, GrubKey,
    GrubMenu, GrubPlatform, GrubSetting, GrubSettings, GrubState, GrubTimeout, InvalidGrubSettings,
    InvalidKernelParameters, KernelParameters, MenuEntry,
};
pub use locale::{
    LOCALE_CONF, LOCALE_GEN, LocaleEntry, is_builtin_locale, same_locale, set_system_lang,
    system_lang,
};
pub use merge::{
    CopyKind, InvalidMergeCopy, MergeChoice, MergeCopy, MergeResolve, PACKAGE_FILES, PACMAN_LOCAL,
    Resolution, SNAPPER_ROOT_CONFIG, UnfitChoice, lists_backup, snapshot as merge_snapshot,
};
pub use mirrorlist::{InvalidMirrorlist, Mirrorlist, SAMPLE_MIRRORS};
pub use name::{
    ADMIN_GROUPS, BootloaderId, BuildRun, Charset, DiskDevice, DkmsModuleName, DkmsVersion,
    GroupName, GrubPicture, GrubThemeFile, HostName, InvalidName, KernelModule, KernelParameter,
    KernelRelease, KnownPackage, LoaderDefault, LoaderEntryId, LocaleName, MenuEntryId, Name,
    NameKind, PackageName, PartitionDevice, PlymouthTheme, Reason, Rule, UnitName, UserName,
    VolumeDevice,
};
pub use network::{
    DEFAULT_HOPS, DEFAULT_PING_TIMEOUT, DEFAULT_PINGS, DEFAULT_TEST_HOST, HopLimit,
    InvalidMacAddress, MODPROBE_BLACKLIST, MacAddress, PingCount, PingTimeout, blacklist_entry,
    blacklisted_modules, set_blacklisted,
};
pub use page_name::{InvalidPageName, MAX_PAGE_NAME_BYTES, PageName};
pub use plan::{Plan, PlannedAction, Segment, Step, StepId};
pub use protected::{PROTECTED, Protection, protection};
pub use repair::{
    ESP_FOLDER, InstalledLoader, InstalledSystem, REPAIR_ROOT, RepairMount, RepairNeed, RepairRoot,
    RepairWork, RootLayout,
};
pub use scheme::{InvalidRgb, InvalidSchemeId, MAX_SCHEME_ID_BYTES, Rgb, SchemeId};
pub use sdboot::EspPlace;
pub use secret::{
    InvalidPassword, InvalidSecretLine, MAX_PASSWORD_BYTES, Password, SecretInput,
    parse_secret_line,
};
pub use settings::{
    DNS_CONFIG, DnsProvider, NETWORKMANAGER_CONF_D, OLD_WIFI_BACKEND_CONFIG, WIFI_BACKEND_CONFIG,
    WifiBackend,
};
pub use shell::{InvalidShellPath, ShellPath};
pub use toggle::{Toggle, option_as_bool};
pub use user_config::{
    ConfigReload, ConfigWord, EditError, HomePath, InvalidConfigWord, InvalidHomePath, Rewrite,
    UserConfigEdit,
};
