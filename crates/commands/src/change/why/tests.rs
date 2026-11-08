use toolbox_core::features::boot::Refusal as BootRefusal;
use toolbox_core::features::boot::edit::EditProblem;
use toolbox_core::features::bootrepair::Refusal as RepairRefusal;
use toolbox_core::features::locales::Refusal as LocaleRefusal;
use toolbox_core::features::network::Refusal as DriverRefusal;
use toolbox_core::features::users::Refusal as AccountRefusal;
use toolbox_plan::GrubPicture;

use super::{SWITCH_UNKNOWN, SWITCH_UNMET, account, boot_edit, driver, locale, membership, repair};

fn one_line(text: &str) {
    assert!(!text.contains('\n'), "{text}");
    assert!(!text.contains(';'), "{text}");
    assert!(!text.contains("see the window"), "{text}");
    assert!(
        text.chars()
            .next()
            .is_some_and(|first| !first.is_uppercase()),
        "starts lower-case: {text}"
    );
}

#[test]
fn every_refusal_has_its_own_english_line() {
    let repairs = [
        RepairRefusal::NotOffered,
        RepairRefusal::Unavailable,
        RepairRefusal::UnknownSystem,
        RepairRefusal::InUse,
        RepairRefusal::UnknownTarget,
        RepairRefusal::UnknownLayout,
        RepairRefusal::NoPassphrase,
    ];

    let accounts = [
        AccountRefusal::Unavailable,
        AccountRefusal::UnknownUser,
        AccountRefusal::UnknownGroup,
        AccountRefusal::Root,
        AccountRefusal::CurrentUser,
        AccountRefusal::SystemAccount,
        AccountRefusal::LastAdmin,
        AccountRefusal::LastAdminLock,
        AccountRefusal::LockStateUnknown,
        AccountRefusal::PrimaryGroup,
        AccountRefusal::SystemGroup,
        AccountRefusal::UserExists,
        AccountRefusal::GroupExists,
        AccountRefusal::ShellNotListed,
        AccountRefusal::UidInUse,
        AccountRefusal::UidOutOfRange,
        AccountRefusal::HomeTaken,
    ];

    let locales = [
        LocaleRefusal::SystemLocale,
        LocaleRefusal::LastLocale,
        LocaleRefusal::NotEnabled,
    ];

    let drivers = [DriverRefusal::OtherFile, DriverRefusal::Unavailable];
    let boots = [
        BootRefusal::Unavailable,
        BootRefusal::CustomCommandLine,
        BootRefusal::TooManyParameters,
        BootRefusal::UnknownTheme,
    ];

    let lines: Vec<String> = repairs
        .map(|refusal| repair(refusal).to_owned())
        .into_iter()
        .chain(accounts.map(|refusal| account(refusal).to_owned()))
        .chain(locales.map(|refusal| locale(refusal).to_owned()))
        .chain(drivers.map(|refusal| driver(refusal).to_owned()))
        .chain(boots.map(|refusal| boot_edit(EditProblem::Refused(refusal), "x")))
        .chain([SWITCH_UNKNOWN.to_owned(), SWITCH_UNMET.to_owned()])
        .collect();

    for line in &lines {
        one_line(line);
    }

    let mut unique = lines.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), lines.len(), "{lines:?}");
}

#[test]
fn a_system_on_a_disk_in_use_says_so() {
    assert_eq!(
        repair(RepairRefusal::InUse),
        "the running system uses this disk, so nothing is written to it"
    );
}

#[test]
fn the_last_admin_says_why_it_stays() {
    assert_eq!(
        account(AccountRefusal::LastAdmin),
        "this is the last member of the groups wheel and sudo, \
         without one nobody could administer this system"
    );
}

#[test]
fn a_bad_boot_value_names_the_value() {
    let invalid = GrubPicture::new("/boot/../x.png").unwrap_err();
    let line = boot_edit(EditProblem::Invalid(invalid), "/boot/../x.png");
    one_line(&line);
    assert_eq!(line, "`/boot/../x.png` is not a full path to a file");

    let theme = boot_edit(EditProblem::Refused(BootRefusal::UnknownTheme), "spin");
    assert_eq!(theme, "`spin` is not an installed Plymouth theme");

    assert_eq!(
        boot_edit(EditProblem::OutOfRange, "9000"),
        "`9000` cannot be used here"
    );
}

#[test]
fn a_membership_names_the_account_it_keeps() {
    let line = membership(AccountRefusal::SystemGroup, "docker");
    one_line(&line);
    assert_eq!(
        line,
        "`docker`: this is a system group, installed programs rely on it"
    );
}
