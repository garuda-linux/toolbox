use toolbox_cli::args::Confirm;
use toolbox_cli::output::{Paint, Style};
use toolbox_plan::{
    PackageName, Plan, PlannedAction, RootAction, ShellPath, TargetList, UserAction, UserName,
};

use super::{planned, planned_out};
use crate::out::{Out, Settings};

fn shell_change() -> Plan {
    Plan::new([PlannedAction::Root(RootAction::SetShell {
        user: UserName::new("ada").unwrap(),
        shell: ShellPath::new("/usr/bin/fish").unwrap(),
    })])
}

#[test]
fn each_part_of_a_step_has_its_own_line_under_the_first() {
    assert_eq!(
        planned(Paint::new(Style::Plain), &shell_change()),
        "Planned  (1 step)\n\n  \
         check that /usr/bin/fish is listed in /etc/shells\n    \
         chsh -s /usr/bin/fish -- ada\n"
    );
}

#[test]
fn a_step_of_one_part_is_one_line() {
    let plan = Plan::new([PlannedAction::Root(RootAction::RefreshKeyring)]);
    let text = planned(Paint::new(Style::Plain), &plan);

    assert_eq!(text.lines().count(), 3, "{text}");
}

#[test]
fn a_review_stays_in_the_printed_plan_under_yes() {
    let bases = TargetList::new([PackageName::new("octopi-git").unwrap()]).unwrap();
    let plan = Plan::new([PlannedAction::User(UserAction::ReviewAur { bases })]);
    let confirm = Confirm {
        yes: true,
        dry_run: false,
        json: false,
    };

    let mut out = Vec::new();

    let exit = planned_out(
        Paint::new(Style::Plain),
        &plan,
        confirm,
        &mut Out::new(&mut out, Settings::default()),
    );

    assert_eq!(exit, None);
    assert!(String::from_utf8(out).unwrap().contains("octopi-git"));
}
