use toolbox_cli::output::{Paint, Style};
use toolbox_plan::{Plan, PlannedAction, RootAction, ShellPath, UserName};

use super::planned;

fn shell_change() -> Plan {
    Plan::new([PlannedAction::Root(RootAction::SetShell {
        user: UserName::new("nico").unwrap(),
        shell: ShellPath::new("/usr/bin/fish").unwrap(),
    })])
}

#[test]
fn each_part_of_a_step_has_its_own_line_under_the_first() {
    assert_eq!(
        planned(Paint::new(Style::Plain), &shell_change()),
        "Planned  (1 step)\n\n  \
         check that /usr/bin/fish is listed in /etc/shells\n    \
         chsh -s /usr/bin/fish -- nico\n"
    );
}

#[test]
fn a_step_of_one_part_is_one_line() {
    let plan = Plan::new([PlannedAction::Root(RootAction::RefreshKeyring)]);
    let text = planned(Paint::new(Style::Plain), &plan);

    assert_eq!(text.lines().count(), 3, "{text}");
}
