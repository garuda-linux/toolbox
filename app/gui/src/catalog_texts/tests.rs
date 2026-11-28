use std::path::PathBuf;

use toolbox_catalog::{Check, FileProblem, Problem};
use toolbox_plan::{
    ConfigWord, GroupName, HomePath, InvalidConfigFilePath, InvalidConfigText, InvalidConfigWord,
    InvalidHomePath, InvalidName, InvalidRgb, NameKind, PackageName, Reason, UnitName,
    UserConfigEdit,
};

use super::{file, problem_text};
use crate::i18n::set_language;

/// The prefixes of the ids a missing text would show instead of its words.
const RAW_ID_PREFIXES: [&str; 5] = [
    "catalog-problem-",
    "catalog-check-",
    "catalog-error-",
    "name-kind-",
    "name-reason-",
];

fn checks_of_every_kind() -> Vec<Check> {
    vec![
        Check::Package(PackageName::new("linux").expect("a package name")),
        Check::Service(UnitName::new("cups.service").expect("a unit name")),
        Check::UserService(UnitName::new("pipewire.service").expect("a unit name")),
        Check::Group(GroupName::new("wheel").expect("a group name")),
        Check::UserConfig(UserConfigEdit::JsonFlag {
            path: HomePath::new(".config/kdeglobals.json").expect("a home path"),
            key: ConfigWord::new("flag").expect("a config word"),
        }),
    ]
}

fn plain_problems() -> Vec<Problem> {
    let mut all = vec![Problem::BadId, Problem::DuplicateId, Problem::EmptyName];
    all.extend(vec![
        Problem::EmptyTitle,
        Problem::EmptyDescription,
        Problem::EmptyIcon,
    ]);
    all.extend(vec![
        Problem::NoEntries,
        Problem::EmptyRequirement,
        Problem::RequiresItself,
    ]);
    all.extend(vec![
        Problem::NoFiles,
        Problem::ReadOnlyHome,
        Problem::EmptyContent,
    ]);
    all.extend(vec![
        Problem::ReloadWithoutUserConfig,
        Problem::RestartWithoutUserConfig,
    ]);
    all.extend(vec![
        Problem::EmptyDesktop,
        Problem::AlsoWithoutPackage,
        Problem::EmptyAlso,
    ]);
    all.extend(vec![
        Problem::EmptyConflicts,
        Problem::ConflictsItself,
        Problem::EmptyLicence,
    ]);

    all
}

fn valued_problems() -> Vec<Problem> {
    let mut all = vec![
        Problem::DuplicatePath("/etc/x".to_owned()),
        Problem::AdminGroup("wheel".to_owned()),
        Problem::HiddenCharacter("a\u{202e}b".to_owned()),
    ];

    all.push(Problem::UnknownRequirement("cups".to_owned()));
    all.push(Problem::RequirementRepeats("cups".to_owned()));
    all.push(Problem::RequirementCycle(vec![
        "a".to_owned(),
        "b".to_owned(),
    ]));
    all.push(Problem::OutsideSkel("etc/x".to_owned()));
    all.push(Problem::ParentComponent("../x".to_owned()));
    all.push(Problem::NotWritable("/etc/x".to_owned()));
    all.push(Problem::ReadOnlyWritable("/etc/y".to_owned()));
    all.push(Problem::UnknownReload("plasma".to_owned()));
    all.push(Problem::RestartRepeats("x.service".to_owned()));
    all.push(Problem::AlsoRepeats("vim".to_owned()));
    all.push(Problem::ConflictRepeats("a".to_owned()));
    all.push(Problem::UnknownConflict("b".to_owned()));
    all.push(Problem::ConflictsRequirement("c".to_owned()));
    all.push(Problem::BadIcon("a/b".to_owned()));
    all.push(Problem::MissingIcon("x.png".to_owned()));
    all.push(Problem::BadGlyph("Bad".to_owned()));
    all.push(Problem::ProtectedAlso("linux".to_owned()));
    all.push(Problem::BadUrl("http://x".to_owned()));
    all.push(Problem::AnsiCount(3));
    all.push(Problem::BadColour(InvalidRgb("red".to_owned())));

    all
}

fn wrapped_problems() -> Vec<Problem> {
    vec![
        Problem::BadName(InvalidName::new(
            NameKind::Package,
            "a b",
            Reason::InvalidCharacter(' '),
        )),
        Problem::BadHomePath(InvalidHomePath::Empty),
        Problem::BadHomePath(InvalidHomePath::TooLong { max: 1024 }),
        Problem::BadHomePath(InvalidHomePath::Absolute("/etc".to_owned())),
        Problem::BadHomePath(InvalidHomePath::InvalidCharacter('\n')),
        Problem::BadHomePath(InvalidHomePath::InvalidComponent("a/../b".to_owned())),
        Problem::BadConfigWord(InvalidConfigWord::Empty),
        Problem::BadConfigWord(InvalidConfigWord::TooLong { max: 255 }),
        Problem::BadConfigWord(InvalidConfigWord::InvalidCharacter('\t')),
        Problem::BadConfigWord(InvalidConfigWord::Padded(" a".to_owned())),
        Problem::BadConfigText(InvalidConfigText::TooLarge { max: 1 }),
        Problem::BadConfigText(InvalidConfigText::NotText),
        Problem::BadConfigPath(InvalidConfigFilePath::Empty),
        Problem::BadConfigPath(InvalidConfigFilePath::TooLong { max: 4096 }),
        Problem::BadConfigPath(InvalidConfigFilePath::Relative("etc".to_owned())),
        Problem::BadConfigPath(InvalidConfigFilePath::InvalidCharacter('\0')),
        Problem::BadConfigPath(InvalidConfigFilePath::InvalidComponent("/a/..".to_owned())),
        Problem::BadConfigPath(InvalidConfigFilePath::OutsideRoots("/home".to_owned())),
        Problem::BadConfigPath(InvalidConfigFilePath::Withheld("/etc/shadow".to_owned())),
    ]
}

/// Every variant of [`Problem`], with each nested value's variants too.
fn samples() -> Vec<Problem> {
    let mut all = plain_problems();
    all.extend(valued_problems());
    all.extend(
        checks_of_every_kind()
            .into_iter()
            .map(|check| Problem::DuplicateCheck(Box::new(check))),
    );
    all.extend(wrapped_problems());

    all
}

fn files_of_every_kind() -> Vec<FileProblem> {
    vec![
        FileProblem::Read {
            path: PathBuf::from("/usr/share/garuda-toolbox/catalog/gaming.toml"),
            error: "entity not found".to_owned(),
        },
        FileProblem::Syntax {
            source_name: "gaming.toml".to_owned(),
            error: "expected a value".to_owned(),
        },
        FileProblem::Invalid {
            source_name: "gaming.toml".to_owned(),
            id: "steam".to_owned(),
            problem: Problem::EmptyTitle,
        },
    ]
}

fn shows_raw_id(text: &str) -> bool {
    RAW_ID_PREFIXES.iter().any(|prefix| text.contains(prefix))
}

/// Runs `check` with each language set on its own context, the English
/// one first.
fn in_each_language(check: impl Fn(&egui::Context, &str)) {
    for language in ["en-US", "de-DE"] {
        let ctx = egui::Context::default();
        set_language(&ctx, &language.parse().unwrap());
        check(&ctx, language);
    }
}

/// A sentence without its text would show the bare id, in each language.
#[test]
fn every_problem_has_its_text_in_each_language() {
    in_each_language(|ctx, language| {
        for problem in samples() {
            let text = problem_text(ctx, &problem);

            assert!(
                !text.is_empty() && !shows_raw_id(&text),
                "{language}: {text}"
            );
        }
    });
}

#[test]
fn every_file_problem_has_its_text_in_each_language() {
    in_each_language(|ctx, language| {
        for failed in files_of_every_kind() {
            let text = file(ctx, &failed);

            assert!(!shows_raw_id(&text), "{language}: {text}");
        }
    });
}

#[test]
fn an_invalid_entry_names_its_file_and_id() {
    let ctx = egui::Context::default();
    let failed = FileProblem::Invalid {
        source_name: "gaming.toml".to_owned(),
        id: "steam".to_owned(),
        problem: Problem::EmptyTitle,
    };

    assert_eq!(
        file(&ctx, &failed),
        "gaming.toml: entry \"steam\": empty title"
    );
}

#[test]
fn an_unknown_requirement_reads_in_german() {
    let ctx = egui::Context::default();
    set_language(&ctx, &"de-DE".parse().unwrap());
    let problem = Problem::UnknownRequirement("cups".to_owned());

    assert_eq!(
        problem_text(&ctx, &problem),
        "benötigt „cups“, das ist kein Eintrag dieses Katalogs"
    );
}

#[test]
fn a_cycle_reads_back_to_its_first_entry() {
    let ctx = egui::Context::default();
    let problem = Problem::RequirementCycle(vec!["a".to_owned(), "b".to_owned()]);

    assert_eq!(
        problem_text(&ctx, &problem),
        "the requirements form a cycle: a -> b -> a"
    );
}

/// The parser quotes the broken line of the file, which is outside text.
#[test]
fn hidden_characters_of_a_broken_file_stay_out_of_the_problem_text() {
    let ctx = egui::Context::default();
    let failed = FileProblem::Syntax {
        source_name: "gaming.toml".to_owned(),
        error: "expected a value near \u{1b}[2J\u{202e}steam".to_owned(),
    };

    let shown = file(&ctx, &failed);

    assert!(!shown.contains('\u{1b}'), "{shown:?}");
    assert!(!shown.contains('\u{202e}'), "{shown:?}");
    assert!(shown.contains("steam"));
}
