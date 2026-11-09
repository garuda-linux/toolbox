//! The Boot options page's root actions in the Arch test image, against
//! a fixture `/etc/default/grub` (the image has no GRUB, so the file is
//! put there and removed again). `grub-mkconfig` and
//! `plymouth-set-default-theme` only run where they are installed: the
//! image has neither, so there the tests check that their steps fail
//! without touching anything.

use std::error::Error;
use std::fs;
use std::path::Path;

use toolbox_plan::protocol::Outcome;
use toolbox_plan::{
    DefaultEntry, EFI_FIRMWARE, GrubMenu, GrubPicture, GrubPlatform, GrubSetting, GrubSettings,
    GrubTimeout, KernelParameters, MenuEntryId, PlymouthTheme, RootAction,
};

use super::common::TestResult;
use super::{apply, in_test_image, restore, saved};

const DEFAULTS: &str = "/etc/default/grub";
const BACKUP: &str = "/etc/default/grub.bak";
const MENU: &str = "/boot/grub/grub.cfg";
const MKCONFIG: &str = "/usr/bin/grub-mkconfig";
const SET_THEME: &str = "/usr/bin/plymouth-set-default-theme";

const FIXTURE: &str = "\
# GRUB boot loader configuration

GRUB_DEFAULT=0
GRUB_TIMEOUT=5
GRUB_DISTRIBUTOR=\"Arch\"
GRUB_CMDLINE_LINUX_DEFAULT=\"loglevel=3 quiet\"
GRUB_CMDLINE_LINUX=\"\"

# Uncomment one of them for the gfx desired, a image background or a gfxtheme
#GRUB_BACKGROUND=\"/path/to/wallpaper\"
#GRUB_THEME=\"/path/to/gfxtheme\"

# Uncomment to make GRUB remember the last selection.
#GRUB_SAVEDEFAULT=true
GRUB_DISABLE_SUBMENU=y
GRUB_DISABLE_SUBMENU=n # the last one counts
";

const EDITED: &str = "\
# GRUB boot loader configuration

GRUB_DEFAULT=\"gnulinux-advanced-1>gnulinux-linux-lts-1\"
GRUB_TIMEOUT=12
GRUB_DISTRIBUTOR=\"Arch\"
GRUB_CMDLINE_LINUX_DEFAULT=\"loglevel=3 quiet splash mitigations=off\"
GRUB_CMDLINE_LINUX=\"\"

# Uncomment one of them for the gfx desired, a image background or a gfxtheme
GRUB_BACKGROUND=\"/usr/share/toolbox-test/it's.png\"
#GRUB_THEME=\"/path/to/gfxtheme\"

# Uncomment to make GRUB remember the last selection.
GRUB_SAVEDEFAULT=true
GRUB_DISABLE_SUBMENU=y
GRUB_DISABLE_SUBMENU=y # the last one counts
GRUB_DISABLE_OS_PROBER=false
";

fn settings() -> Result<GrubSettings, Box<dyn Error>> {
    Ok(GrubSettings::new(vec![
        GrubSetting::Default(DefaultEntry::Entry(MenuEntryId::new(
            "gnulinux-advanced-1>gnulinux-linux-lts-1",
        )?)),
        GrubSetting::Timeout(GrubTimeout::new(12)?),
        GrubSetting::KernelParameters(
            KernelParameters::parse("loglevel=3 quiet splash mitigations=off")
                .ok_or("no parameters")?,
        ),
        GrubSetting::Submenus(false),
        GrubSetting::SaveDefault(true),
        GrubSetting::OsProber(true),
        GrubSetting::Background(Some(GrubPicture::new("/usr/share/toolbox-test/it's.png")?)),
    ])?)
}

/// The typed edit as root on the real `/etc`: only the wanted values
/// change, the old file stays beside it, a repeat writes nothing.
#[test]
fn grub_settings_change_only_their_lines_of_the_real_file() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let (defaults, backup) = (saved(DEFAULTS)?, saved(BACKUP)?);

    fs::create_dir_all("/etc/default")?;
    fs::write(DEFAULTS, FIXTURE)?;

    let first = apply(vec![RootAction::SetGrubSettings {
        settings: settings()?,
    }])?;

    let written = fs::read_to_string(DEFAULTS)?;
    let kept = fs::read_to_string(BACKUP)?;
    let second = apply(vec![RootAction::SetGrubSettings {
        settings: settings()?,
    }])?;

    let sourced = std::process::Command::new("/usr/bin/sh")
        .args([
            "-c",
            ". /etc/default/grub && printf '%s|%s|%s' \"$GRUB_DEFAULT\" \"$GRUB_BACKGROUND\" \"$GRUB_CMDLINE_LINUX_DEFAULT\"",
        ])
        .output()?;

    restore(DEFAULTS, defaults.as_deref())?;
    restore(BACKUP, backup.as_deref())?;

    assert_eq!(first, [Outcome::Succeeded]);
    assert_eq!(written, EDITED);
    assert_eq!(kept, FIXTURE);
    assert_eq!(second, [Outcome::Skipped], "already as wanted");

    // A real shell reads the values back as they were asked for.
    assert_eq!(
        String::from_utf8_lossy(&sourced.stdout),
        "gnulinux-advanced-1>gnulinux-linux-lts-1|/usr/share/toolbox-test/it's.png|loglevel=3 quiet splash mitigations=off"
    );
    Ok(())
}

/// Without `/etc/default/grub` GRUB is not the boot loader: the step
/// fails and the file is not created.
#[test]
fn grub_settings_fail_without_the_file_and_create_nothing() -> TestResult {
    if !in_test_image() || Path::new(DEFAULTS).exists() {
        return Ok(());
    }

    let outcomes = apply(vec![
        RootAction::SetGrubSettings {
            settings: settings()?,
        },
        RootAction::GenerateGrubConfig,
    ])?;

    assert!(
        matches!(
            outcomes.as_slice(),
            [Outcome::Failed {
                cause: _,
                cleanup_failed: _
            }]
        ),
        "the plan must stop before grub-mkconfig: {outcomes:?}"
    );
    assert!(!Path::new(DEFAULTS).exists());
    Ok(())
}

/// Runs `grub-mkconfig` only where GRUB is installed and has a menu already.
/// Without a menu it would write one into an image that boots nothing, so
/// that case is skipped before anything runs.
#[test]
fn generating_the_menu_needs_grub() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let tools = Path::new(MKCONFIG).exists();
    let menu = Path::new(MENU).exists();

    if tools && !menu {
        return Ok(());
    }

    let outcomes = apply(vec![RootAction::GenerateGrubConfig])?;

    if tools {
        assert_eq!(outcomes, [Outcome::Succeeded]);
        assert_ne!(
            GrubMenu::for_platform(
                &fs::read_to_string(MENU)?,
                GrubPlatform::from_efi_firmware(Path::new("/").join(EFI_FIRMWARE).is_dir())
            )
            .entries
            .len(),
            0
        );
    } else {
        assert!(
            matches!(
                outcomes.as_slice(),
                [Outcome::Failed {
                    cause: _,
                    cleanup_failed: _
                }]
            ),
            "{outcomes:?}"
        );
        assert!(!Path::new(MENU).exists(), "a menu appeared without GRUB");
    }

    Ok(())
}

/// Only where Plymouth is installed, with the theme it has now, so the
/// run rebuilds the initramfs and changes nothing else. Elsewhere the
/// step must fail.
#[test]
fn the_plymouth_theme_needs_plymouth() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    if !Path::new(SET_THEME).exists() {
        let outcomes = apply(vec![RootAction::SetPlymouthTheme {
            theme: PlymouthTheme::new("bgrt")?,
        }])?;

        assert!(
            matches!(
                outcomes.as_slice(),
                [Outcome::Failed {
                    cause: _,
                    cleanup_failed: _
                }]
            ),
            "{outcomes:?}"
        );
        return Ok(());
    }

    let current = std::process::Command::new(SET_THEME).output()?;
    let theme = PlymouthTheme::new(String::from_utf8_lossy(&current.stdout).trim())?;

    assert_eq!(
        apply(vec![RootAction::SetPlymouthTheme { theme }])?,
        [Outcome::Succeeded]
    );
    Ok(())
}

#[test]
fn the_menu_read_answers_titles_and_ids_only() -> TestResult {
    if !in_test_image() || Path::new(MENU).exists() {
        return Ok(());
    }

    let made_folder = !Path::new("/boot/grub").exists();

    fs::create_dir_all("/boot/grub")?;
    fs::write(
        MENU,
        "password_pbkdf2 admin grub.pbkdf2.sha512.10000.ABCDEF\n\
         menuentry 'Arch Linux' $menuentry_id_option 'gnulinux-simple-1' {\n\
         \tlinux /vmlinuz-linux root=UUID=1 rw\n}\n",
    )?;

    let output = crate::oneshot::output(&["read", "grub-menu"])?;

    fs::remove_file(MENU)?;

    if made_folder {
        fs::remove_dir("/boot/grub")?;
    }

    let line = String::from_utf8(output.stdout)?;
    let menu = GrubMenu::parse(line.trim_end())?;

    assert!(output.status.success());
    assert_eq!(menu.entries.len(), 1);
    assert_eq!(
        menu.entries.first().map(|entry| entry.id.as_str()),
        Some("gnulinux-simple-1")
    );
    assert!(
        !line.contains("pbkdf2") && !line.contains("vmlinuz"),
        "{line}"
    );
    Ok(())
}
