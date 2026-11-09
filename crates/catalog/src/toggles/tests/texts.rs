//! A title or description reads on the page as it reads in the file.

use super::{SECTION, problem_of};
use crate::Problem;

fn entry_with(title: &str, description: &str) -> String {
    format!(
        "{SECTION}[[section.entry]]\nid = \"a\"\ntitle = \"{title}\"\ndescription = \"{description}\"\ncheck = {{ package = \"a\" }}\n"
    )
}

#[test]
fn a_bidi_override_in_a_title_is_refused() {
    let title = "Steam \u{202e}evomer";

    assert_eq!(
        problem_of(&entry_with(title, "D")).2,
        Problem::HiddenCharacter(title.to_owned())
    );
}

#[test]
fn a_zero_width_joiner_in_a_description_is_refused() {
    let description = "Runs\u{200d} games";

    assert_eq!(
        problem_of(&entry_with("T", description)).2,
        Problem::HiddenCharacter(description.to_owned())
    );
}
