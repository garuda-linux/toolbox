//! The Locales page's root actions in the Arch test image: the typed edit
//! of `/etc/locale.gen` with a real `locale-gen`, and the system language
//! in `/etc/locale.conf`. DKMS is not in the image, its command is covered
//! by the argv test.
//!
//! The official Arch image leaves out most of glibc's locale definitions
//! (`NoExtract` in its `pacman.conf`), so the test takes the first locale
//! the image can generate.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use toolbox_plan::protocol::{Mode, Outcome};
use toolbox_plan::{LocaleEntry, LocaleName, RootAction, Toggle};

use super::common::{TestResult, helper, request};
use super::{apply, in_test_image, printed, restore, saved};

const LOCALE_GEN: &str = "/etc/locale.gen";
const LOCALE_CONF: &str = "/etc/locale.conf";
const DEFINITIONS: &str = "/usr/share/i18n/locales";
const UTF8_CHARMAP: &str = "/usr/share/i18n/charmaps/UTF-8.gz";
const CANDIDATES: [&str; 3] = ["de_DE", "en_GB", "en_US"];

fn spare_locale(locale_gen: &str) -> Option<LocaleEntry> {
    if !Path::new(UTF8_CHARMAP).exists() {
        return None;
    }

    let enabled = LocaleEntry::enabled_in(locale_gen);

    CANDIDATES
        .into_iter()
        .filter(|name| Path::new(DEFINITIONS).join(name).is_file())
        .filter_map(|name| LocaleEntry::parse(&format!("{name}.UTF-8 UTF-8")))
        .find(|entry| !enabled.contains(entry))
}

fn generated() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("locale").arg("-a").output()?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect())
}

/// `de_DE.UTF-8` shows as `de_DE.utf8` in `locale -a`.
fn listed_name(entry: &LocaleEntry) -> String {
    entry.locale.as_str().replace(".UTF-8", ".utf8")
}

fn switch(entry: &LocaleEntry, enabled: bool) -> RootAction {
    RootAction::SetLocaleEnabled {
        locale: entry.clone(),
        enabled,
    }
}

#[test]
fn an_enabled_locale_is_generated_and_a_disabled_one_is_gone() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let file = saved(LOCALE_GEN)?;
    let before = String::from_utf8(file.clone().unwrap_or_default())?;
    let Some(entry) = spare_locale(&before) else {
        return Err(format!(
            "none of {} is defined in {DEFINITIONS}: the image lacks glibc's locale data",
            CANDIDATES.join(", ")
        )
        .into());
    };

    let name = listed_name(&entry);

    assert_eq!(
        apply(vec![switch(&entry, true), RootAction::GenerateLocales])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );

    let enabled = fs::read_to_string(LOCALE_GEN)?;

    assert_eq!(
        enabled,
        entry.set(&before, Toggle::On),
        "only that line changes"
    );
    assert!(generated()?.contains(&name), "{:?}", generated()?);

    assert_eq!(
        apply(vec![switch(&entry, true)])?,
        [Outcome::Skipped],
        "already on"
    );

    assert_eq!(
        apply(vec![switch(&entry, false), RootAction::GenerateLocales])?,
        [Outcome::Succeeded, Outcome::Succeeded]
    );
    assert_eq!(
        fs::read_to_string(LOCALE_GEN)?,
        entry.set(&enabled, Toggle::Off)
    );
    assert!(!generated()?.contains(&name), "{:?}", generated()?);

    restore(LOCALE_GEN, file.as_deref())?;
    assert_eq!(
        apply(vec![RootAction::GenerateLocales])?,
        [Outcome::Succeeded]
    );
    Ok(())
}

#[test]
fn the_system_locale_is_written_only_for_an_enabled_locale() -> TestResult {
    if !in_test_image() {
        return Ok(());
    }

    let saved_gen = saved(LOCALE_GEN)?;
    let saved_conf = saved(LOCALE_CONF)?;
    let system = |name: &str| -> Result<RootAction, Box<dyn Error>> {
        Ok(RootAction::SetSystemLocale {
            locale: LocaleName::new(name)?,
        })
    };

    // The builtin reads the file, not the generated locales, so no
    // locale-gen is needed here.
    fs::write(LOCALE_GEN, "#de_DE.UTF-8 UTF-8\nen_US.UTF-8 UTF-8\n")?;
    fs::write(LOCALE_CONF, "LANG=C\nLC_TIME=C\n")?;

    let refused = helper(
        &["apply"],
        &request(Mode::Run, vec![system("de_DE.UTF-8")?])?,
    )?;

    assert!(!refused.status.success());
    assert!(
        printed(&refused)?.contains("is not enabled in /etc/locale.gen"),
        "{}",
        printed(&refused)?
    );
    assert_eq!(fs::read_to_string(LOCALE_CONF)?, "LANG=C\nLC_TIME=C\n");

    assert_eq!(apply(vec![system("en_US.UTF-8")?])?, [Outcome::Succeeded]);
    assert_eq!(
        fs::read_to_string(LOCALE_CONF)?,
        "LANG=en_US.UTF-8\nLC_TIME=C\n"
    );
    assert_eq!(
        apply(vec![system("en_US.utf8")?])?,
        [Outcome::Succeeded],
        "another spelling is written as asked"
    );
    assert_eq!(
        apply(vec![system("en_US.utf8")?])?,
        [Outcome::Skipped],
        "already set"
    );

    restore(LOCALE_GEN, saved_gen.as_deref())?;
    restore(LOCALE_CONF, saved_conf.as_deref())?;
    Ok(())
}
