//! The refusals of each name kind with the reason it gives, so a rule
//! that changes its reason or stops refusing fails here.

use crate::{
    Charset, DkmsModuleName, DkmsVersion, KernelModule, KernelRelease, LoaderDefault,
    LoaderEntryId, LocaleName, PackageName, PlymouthTheme, Reason, UnitName, UserName,
};

fn run(cases: Vec<(String, Reason)>, check: impl Fn(&str) -> Reason) {
    for (input, reason) in cases {
        assert_eq!(check(&input), reason, "{input:?}");
    }
}

#[test]
fn package_names_refuse_hidden_and_long_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("vim\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("vim\nneovim".to_owned(), Reason::InvalidCharacter('\n')),
            ("vim\tneovim".to_owned(), Reason::InvalidCharacter('\t')),
            ("a".repeat(256), Reason::TooLong { max: 255 }),
        ],
        |input| PackageName::new(input).unwrap_err().reason(),
    );
    PackageName::new("a".repeat(255)).unwrap();
}

#[test]
fn unit_names_refuse_hidden_option_and_long_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a\0.service".to_owned(), Reason::InvalidCharacter('\0')),
            (
                "a\u{1}.service".to_owned(),
                Reason::InvalidCharacter('\u{1}'),
            ),
            (
                "a\u{7f}.service".to_owned(),
                Reason::InvalidCharacter('\u{7f}'),
            ),
            ("-a.service".to_owned(), Reason::InvalidStart('-')),
            (
                format!("{}.service", "a".repeat(256)),
                Reason::TooLong { max: 255 },
            ),
        ],
        |input| UnitName::new(input).unwrap_err().reason(),
    );
}

#[test]
fn user_names_refuse_long_hidden_and_path_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a".repeat(33), Reason::TooLong { max: 32 }),
            ("ada\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("ada\n".to_owned(), Reason::InvalidCharacter('\n')),
            ("ada\u{1b}".to_owned(), Reason::InvalidCharacter('\u{1b}')),
            ("ada/x".to_owned(), Reason::InvalidCharacter('/')),
            ("-ada".to_owned(), Reason::InvalidStart('-')),
        ],
        |input| UserName::new(input).unwrap_err().reason(),
    );
}

#[test]
fn locale_names_refuse_long_hidden_and_path_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a".repeat(65), Reason::TooLong { max: 64 }),
            ("en_US\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("en_US\n".to_owned(), Reason::InvalidCharacter('\n')),
            ("en/US".to_owned(), Reason::InvalidCharacter('/')),
            ("1en_US".to_owned(), Reason::InvalidStart('1')),
        ],
        |input| LocaleName::new(input).unwrap_err().reason(),
    );
}

#[test]
fn charsets_refuse_long_hidden_and_path_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a".repeat(33), Reason::TooLong { max: 32 }),
            ("UTF\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("UTF/8".to_owned(), Reason::InvalidCharacter('/')),
            ("-UTF8".to_owned(), Reason::InvalidStart('-')),
        ],
        |input| Charset::new(input).unwrap_err().reason(),
    );
}

#[test]
fn kernel_modules_refuse_their_length_and_hidden_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a".repeat(56), Reason::TooLong { max: 55 }),
            ("r\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("r\n".to_owned(), Reason::InvalidCharacter('\n')),
            ("r.x".to_owned(), Reason::InvalidCharacter('.')),
            ("-r".to_owned(), Reason::InvalidStart('-')),
        ],
        |input| KernelModule::new(input).unwrap_err().reason(),
    );
    KernelModule::new("a".repeat(55)).unwrap();
}

#[test]
fn dkms_module_names_refuse_long_hidden_and_option_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a".repeat(65), Reason::TooLong { max: 64 }),
            ("nv\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("nv idia".to_owned(), Reason::InvalidCharacter(' ')),
            ("-nvidia".to_owned(), Reason::InvalidStart('-')),
        ],
        |input| DkmsModuleName::new(input).unwrap_err().reason(),
    );
}

#[test]
fn dkms_versions_refuse_long_hidden_and_option_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("1".repeat(65), Reason::TooLong { max: 64 }),
            ("1.0\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("1 0".to_owned(), Reason::InvalidCharacter(' ')),
            ("-k".to_owned(), Reason::InvalidStart('-')),
        ],
        |input| DkmsVersion::new(input).unwrap_err().reason(),
    );
}

#[test]
fn kernel_releases_refuse_long_hidden_and_path_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("6".repeat(65), Reason::TooLong { max: 64 }),
            ("6.1\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("6/1".to_owned(), Reason::InvalidCharacter('/')),
            ("..".to_owned(), Reason::InvalidStart('.')),
        ],
        |input| KernelRelease::new(input).unwrap_err().reason(),
    );
}

#[test]
fn plymouth_themes_refuse_long_hidden_and_option_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a".repeat(65), Reason::TooLong { max: 64 }),
            ("bgrt\0".to_owned(), Reason::InvalidCharacter('\0')),
            ("bgrt/x".to_owned(), Reason::InvalidCharacter('/')),
            ("-x".to_owned(), Reason::InvalidStart('-')),
        ],
        |input| PlymouthTheme::new(input).unwrap_err().reason(),
    );
}

#[test]
fn loader_entry_ids_refuse_long_glob_and_hidden_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a".repeat(256), Reason::TooLong { max: 255 }),
            ("e\n".to_owned(), Reason::InvalidCharacter('\n')),
            ("e*".to_owned(), Reason::InvalidCharacter('*')),
            ("e x".to_owned(), Reason::InvalidCharacter(' ')),
            ("-x".to_owned(), Reason::InvalidStart('-')),
            ("@x".to_owned(), Reason::InvalidStart('@')),
        ],
        |input| LoaderEntryId::new(input).unwrap_err().reason(),
    );
}

#[test]
fn loader_defaults_refuse_long_hidden_and_path_text() {
    run(
        vec![
            (String::new(), Reason::Empty),
            ("a".repeat(256), Reason::TooLong { max: 255 }),
            ("e\n".to_owned(), Reason::InvalidCharacter('\n')),
            ("e/x".to_owned(), Reason::InvalidCharacter('/')),
            ("e%".to_owned(), Reason::InvalidCharacter('%')),
            ("-x".to_owned(), Reason::InvalidStart('-')),
        ],
        |input| LoaderDefault::new(input).unwrap_err().reason(),
    );
    LoaderDefault::new("@saved").unwrap();
}
