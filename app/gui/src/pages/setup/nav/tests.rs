use super::filled;
use crate::pages::setup::Stage;

fn step() -> Stage {
    Stage::Step("audio".to_owned())
}

#[test]
fn step_n_of_m_fills_n_of_m() {
    assert_eq!(filled(&step(), 3, 16), (3, 16));
}

#[test]
fn the_only_step_fills_the_whole_bar() {
    assert_eq!(filled(&step(), 1, 1), (1, 1));
}

#[test]
fn the_welcome_page_fills_none_and_the_summary_all() {
    assert_eq!(filled(&Stage::Welcome, 0, 16), (0, 16));
    assert_eq!(filled(&Stage::Summary, 17, 16), (16, 16));
    assert_eq!(filled(&Stage::Success, 0, 16), (16, 16));
}
