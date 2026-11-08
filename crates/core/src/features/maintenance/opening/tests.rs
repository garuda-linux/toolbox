use super::super::Tool;
use super::{Next, Opening};

#[test]
fn a_tool_opens_once_its_install_ends_well() {
    let opening = Opening::Sent(Next::Tool(Tool::BtrfsAssistant)).started();

    assert_eq!(
        opening.finished(true),
        (Some(Next::Tool(Tool::BtrfsAssistant)), Opening::None)
    );
}

#[test]
fn a_failed_install_opens_nothing() {
    let opening = Opening::Sent(Next::Tool(Tool::BtrfsAssistant)).started();

    assert_eq!(opening.finished(false), (None, Opening::None));
}

#[test]
fn the_end_of_a_plan_that_ran_before_the_install_opens_nothing() {
    let opening = Opening::Sent(Next::Tool(Tool::MergePacdiff));

    assert_eq!(opening.finished(true), (None, opening));
}

#[test]
fn a_refused_install_stops_the_wait() {
    assert_eq!(
        Opening::Sent(Next::Tool(Tool::ArchMirrors)).refused(),
        Opening::None
    );
    assert_eq!(
        Opening::Installing(Next::Tool(Tool::ArchMirrors)).refused(),
        Opening::Installing(Next::Tool(Tool::ArchMirrors))
    );
}

#[test]
fn nothing_waits_without_a_sent_install() {
    assert_eq!(Opening::None.started(), Opening::None);
    assert_eq!(Opening::None.finished(true), (None, Opening::None));
}

#[test]
fn the_mirror_ranking_waits_for_its_program_like_a_tool() {
    let opening = Opening::Sent(Next::RankMirrors).started();

    assert_eq!(
        opening.finished(true),
        (Some(Next::RankMirrors), Opening::None)
    );
}
