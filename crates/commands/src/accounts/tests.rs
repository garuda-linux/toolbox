use toolbox_cli::Exit;
use toolbox_cli::args::{
    AddUser, Confirm, EditUser, GroupArgs, PasswordSource, RemoveUser, UserArgs,
};
use toolbox_cli::confirm::Prompt;
use toolbox_core::domains::{Accounts, AvailableShells, CurrentUser, Group, User};
use toolbox_core::{Snapshot, Toggle};
use toolbox_plan::{GroupName, PlannedAction, RootAction, ShellPath, UserName};

use super::{add, edit, group, lock, remove};

/// Answers with fixed lines: the secret questions get `secrets` in turn,
/// standard input gives `stdin`.
struct Answers {
    terminal: bool,
    secrets: Vec<&'static str>,
    stdin: Option<&'static str>,
}

impl Prompt for Answers {
    fn is_terminal(&self) -> bool {
        self.terminal
    }

    fn ask(&mut self, _question: &str) -> Option<String> {
        None
    }

    fn secret(&mut self, _question: &str) -> Option<String> {
        (!self.secrets.is_empty()).then(|| self.secrets.remove(0).to_owned())
    }

    fn line(&mut self) -> Option<String> {
        self.stdin.map(str::to_owned)
    }
}

fn user(name: &str, uid: u32) -> User {
    User {
        name: UserName::new(name).unwrap(),
        uid,
        gid: uid,
        gecos: String::new(),
        home: format!("/home/{name}"),
        shell: "/usr/bin/bash".parse().unwrap(),
    }
}

fn group_of(name: &str, gid: u32, members: &[&str]) -> Group {
    Group {
        name: GroupName::new(name).unwrap(),
        gid,
        members: members
            .iter()
            .map(|member| UserName::new(*member).unwrap())
            .collect(),
    }
}

fn snapshot() -> Snapshot {
    let mut snapshot = Snapshot::default();
    snapshot.insert(Accounts {
        users: vec![user("ada", 1001), user("nico", 1000), user("root", 0)],
        groups: vec![
            group_of("ada", 1001, &[]),
            group_of("audio", 996, &["ada"]),
            group_of("nico", 1000, &[]),
            group_of("wheel", 998, &["nico"]),
        ],
    });
    snapshot.insert(CurrentUser(UserName::new("nico").unwrap()));
    snapshot.insert(AvailableShells(vec![
        ShellPath::new("/usr/bin/bash").unwrap(),
        ShellPath::new("/usr/bin/fish").unwrap(),
    ]));
    snapshot
}

fn add_args(name: &str) -> AddUser {
    AddUser {
        user: name.parse().unwrap(),
        full_name: Some("Grace Hopper".parse().unwrap()),
        shell: None,
        admin: true,
        groups: Vec::new(),
        password: PasswordSource::default(),
        confirm: Confirm::default(),
    }
}

#[test]
fn a_new_admin_is_created_in_wheel_with_its_password() {
    let mut prompt = Answers {
        terminal: true,
        secrets: vec!["s3cret!\n", "s3cret!\n"],
        stdin: None,
    };

    let plan = add(&add_args("grace"), &snapshot(), &mut prompt).unwrap();

    let PlannedAction::Root(RootAction::CreateUser { user }) = &plan.steps()[0].action else {
        panic!("the first step creates the account");
    };

    assert!(
        user.parts()
            .groups
            .contains(&GroupName::new("wheel").unwrap())
    );

    assert!(matches!(
        plan.steps()[1].action,
        PlannedAction::Root(RootAction::SetPassword {
            user: _,
            password: _
        })
    ));
}

#[test]
fn two_different_passwords_create_nothing() {
    let mut prompt = Answers {
        terminal: true,
        secrets: vec!["one\n", "two\n"],
        stdin: None,
    };

    let refused = add(&add_args("grace"), &snapshot(), &mut prompt).unwrap_err();
    assert!(refused.message().contains("do not match"));
}

#[test]
fn without_a_terminal_the_password_must_come_from_stdin() {
    let mut prompt = Answers {
        terminal: false,
        secrets: Vec::new(),
        stdin: Some("piped\n"),
    };

    add(&add_args("grace"), &snapshot(), &mut prompt).unwrap_err();

    let mut args = add_args("grace");
    args.password.password_stdin = true;
    add(&args, &snapshot(), &mut prompt).unwrap();
}

#[test]
fn a_taken_name_is_refused_before_any_password_is_asked() {
    let mut prompt = Answers {
        terminal: true,
        secrets: Vec::new(),
        stdin: None,
    };

    let refused = add(&add_args("ada"), &snapshot(), &mut prompt).unwrap_err();
    assert_eq!(refused.exit(), Exit::PageRule);
    assert_eq!(refused.message(), "a user by this name exists already");
}

#[test]
fn an_edit_joins_and_leaves_groups_and_changes_the_shell() {
    let args = EditUser {
        user: "ada".parse().unwrap(),
        full_name: None,
        shell: Some("/usr/bin/fish".parse().unwrap()),
        add_groups: vec!["wheel".parse().unwrap()],
        remove_groups: vec!["audio".parse().unwrap()],
        home: None,
        move_files: false,
        confirm: Confirm::default(),
    };

    let plan = edit(&args, &snapshot()).unwrap();
    let text = format!("{:?}", plan.steps());
    assert!(text.contains("wheel"), "joins wheel: {text}");
    assert!(text.contains("audio"), "leaves audio: {text}");
    assert!(text.contains("fish"), "changes the shell: {text}");
}

#[test]
fn an_unlisted_shell_is_refused() {
    let args = EditUser {
        user: "ada".parse().unwrap(),
        full_name: None,
        shell: Some("/usr/bin/zsh".parse().unwrap()),
        add_groups: Vec::new(),
        remove_groups: Vec::new(),
        home: None,
        move_files: false,
        confirm: Confirm::default(),
    };

    edit(&args, &snapshot()).unwrap_err();
}

#[test]
fn the_running_user_cannot_lock_or_remove_itself() {
    let me = UserArgs {
        user: "nico".parse().unwrap(),
        confirm: Confirm::default(),
    };

    lock(&me, Toggle::On, &snapshot()).unwrap_err();

    let removal = RemoveUser {
        user: "nico".parse().unwrap(),
        delete_home: false,
        confirm: Confirm::default(),
    };

    remove(&removal, &snapshot()).unwrap_err();
}

#[test]
fn a_new_group_is_planned() {
    let added = GroupArgs {
        group: "media".parse().unwrap(),
        confirm: Confirm::default(),
    };

    let plan = group(&added, Toggle::On, &snapshot()).unwrap();

    assert_ne!(plan.steps().len(), 0);
}

#[test]
fn an_unknown_account_or_group_hints_at_the_closest_one() {
    let typo = UserArgs {
        user: "adaa".parse().unwrap(),
        confirm: Confirm::default(),
    };

    let refused = lock(&typo, Toggle::On, &snapshot()).unwrap_err();
    assert!(refused.message().ends_with("did you mean `ada`?"));

    let group_typo = GroupArgs {
        group: "audoi".parse().unwrap(),
        confirm: Confirm::default(),
    };

    let group_refused = group(&group_typo, Toggle::Off, &snapshot()).unwrap_err();
    assert!(group_refused.message().ends_with("did you mean `audio`?"));
}
