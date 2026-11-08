use toolbox_plan::KernelParameters;

use super::super::{Flag, add_parameter, free_parameters, is_flag_parameter, parameters, set_flag};
use super::{parameter, queued_line, system};
use crate::{ChangeSet, Toggle};

fn list(words: &str) -> KernelParameters {
    KernelParameters::parse(words).unwrap()
}

#[test]
fn the_free_list_hides_what_the_switches_own() {
    let words = list("loglevel=3 quiet splash mitigations=off sysrq_always_enabled=1 nowatchdog");
    let free = free_parameters(&words);

    assert_eq!(free, ["loglevel=3", "nowatchdog"]);
    assert!(is_flag_parameter("quiet"));
    assert!(is_flag_parameter("sysrq_always_enabled=1"));
    assert!(!is_flag_parameter("nowatchdog"));
}

#[test]
fn a_bare_sysrq_reads_as_the_switch_and_stays_hidden() {
    let words = list("sysrq loglevel=3");

    assert!(Flag::SysRq.is_set(&words));
    assert_eq!(free_parameters(&words), ["loglevel=3"]);
}

#[test]
fn other_mitigations_values_stay_free_and_leave_the_switch_off() {
    let words = list("mitigations=auto,nosmt loglevel=3");

    assert!(!Flag::MitigationsOff.is_set(&words));
    assert!(!is_flag_parameter("mitigations=auto,nosmt"));
    assert_eq!(
        free_parameters(&words),
        ["mitigations=auto,nosmt", "loglevel=3"]
    );
}

#[test]
fn switching_sysrq_off_takes_both_spellings_and_on_unifies_them() {
    let bare = system("GRUB_CMDLINE_LINUX_DEFAULT=\"sysrq loglevel=3\"\n", &[]);
    let mut changes = ChangeSet::default();

    set_flag(Flag::SysRq, Toggle::Off, &mut changes, &bare).unwrap();
    assert_eq!(queued_line(&changes).as_deref(), Some("loglevel=3"));

    let mut switched = ChangeSet::default();

    set_flag(Flag::SysRq, Toggle::On, &mut switched, &bare).unwrap();
    assert_eq!(
        queued_line(&switched).as_deref(),
        Some("loglevel=3 sysrq_always_enabled=1")
    );
}

#[test]
fn switching_a_set_flag_on_keeps_its_word_where_it_is() {
    let state = system("GRUB_CMDLINE_LINUX_DEFAULT=\"quiet loglevel=3\"\n", &[]);
    let mut changes = ChangeSet::default();

    set_flag(Flag::Quiet, Toggle::On, &mut changes, &state).unwrap();

    assert!(changes.is_empty());
}

#[test]
fn a_hand_added_switch_word_queues_the_whole_line() {
    let state = system("GRUB_CMDLINE_LINUX_DEFAULT=\"loglevel=3\"\n", &[]);
    let mut changes = ChangeSet::default();
    add_parameter(parameter("quiet"), &mut changes, &state).unwrap();

    assert_eq!(
        parameters(&state, &changes).unwrap().to_string(),
        "loglevel=3 quiet"
    );
    assert!(Flag::Quiet.is_set(&parameters(&state, &changes).unwrap()));
    assert_eq!(
        free_parameters(&parameters(&state, &changes).unwrap()),
        ["loglevel=3"]
    );
}
