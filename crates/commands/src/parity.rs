//! A new kind without a line here fails the test, so the command line keeps
//! up with the window.

mod tests;

pub const COMMANDS: &[(&str, &[&str])] = &[
    ("account-locks", &["users", "lock", "guest"]),
    ("aur-packages", &["packages", "install", "--aur", "yay-bin"]),
    ("dkms-modules", &["dkms", "build"]),
    (
        "full-names",
        &["users", "edit", "ada", "--full-name", "Ada"],
    ),
    ("grub-settings", &["boot", "set", "timeout", "3"]),
    ("loader-settings", &["boot", "set", "editor", "off"]),
    ("sdboot-kernel", &["boot", "param", "add", "nowatchdog"]),
    ("groups", &["users", "edit", "ada", "--add-group", "wheel"]),
    (
        "home-folders",
        &["users", "edit", "ada", "--home", "/home/lovelace"],
    ),
    ("locales", &["locales", "enable", "de_DE.UTF-8"]),
    (
        "login-shells",
        &["users", "edit", "ada", "--shell", "/usr/bin/fish"],
    ),
    (
        "module-blacklist",
        &["network", "drivers", "blacklist", "r8169"],
    ),
    ("new-groups", &["groups", "add", "family"]),
    ("operations", &["maintenance", "run", "clean-package-cache"]),
    ("packages", &["apps", "install", "firefox"]),
    ("plymouth-theme", &["boot", "set", "plymouth", "bgrt"]),
    ("removed-groups", &["groups", "remove", "media"]),
    ("removed-users", &["users", "remove", "guest"]),
    ("system-locale", &["locales", "system", "de_DE.UTF-8"]),
    ("unit-masks", &["services", "mask", "cups.service"]),
    ("units", &["services", "enable", "sshd.service"]),
    ("user-configs", &["toggle", "on", "kde-borderless-maximize"]),
    (
        "user-unit-masks",
        &["services", "mask", "foo.service", "--user"],
    ),
    (
        "user-units",
        &["services", "enable", "foo.service", "--user"],
    ),
];

/// Domains the window queues that no command does yet, each with why.
pub const NO_COMMAND: &[(&str, &str)] = &[
    ("dns", "set on the System Tweaks page only"),
    ("garuda-update-flags", "set on the System Tweaks page only"),
    ("hblock", "set on the System Tweaks page only"),
    ("wifi-backend", "set on the System Tweaks page only"),
];

pub const AT_ONCE: &[(&str, &[&str])] = &[
    ("add user", &["users", "add", "grace", "--password-stdin"]),
    ("set password", &["users", "password", "ada"]),
    ("service now", &["services", "restart", "sshd.service"]),
    ("update", &["update", "--aur"]),
    ("report", &["diagnostics", "inxi", "--upload"]),
    ("config file", &["configs", "edit", "pacman-config"]),
    ("radio", &["network", "radio", "wifi", "off"]),
    ("public address", &["network", "public-address"]),
    ("ping", &["network", "ping", "garudalinux.org"]),
    ("traceroute", &["network", "traceroute", "garudalinux.org"]),
    ("boot repair", &["bootrepair", "regenerate", "/dev/sda2"]),
    ("grub reinstall", &["bootrepair", "reinstall", "/dev/sda2"]),
    ("setup summary", &["setup", "apply", "firefox", "steam"]),
    ("settings", &["settings", "set", "copy-reports", "off"]),
    ("default kernel", &["kernels", "default", "linux-lts"]),
    (
        "language pack",
        &["locales", "packs", "install", "firefox-i18n-de"],
    ),
];
