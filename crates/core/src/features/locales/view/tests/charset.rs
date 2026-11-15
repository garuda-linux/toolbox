use super::super::LocaleRow;

fn row(code: &str, charset: &str) -> LocaleRow {
    LocaleRow {
        locale_code: code.to_owned(),
        charset: charset.to_owned(),
        language: String::new(),
        territory: String::new(),
        enabled_now: false,
        on: false,
        queued: "",
        system: false,
    }
}

/// Lines of the Lima VM's `SUPPORTED`: a code with `.UTF-8` names its
/// character set, `de_DE`, `de_DE@euro` and `sr_RS@latin` do not.
#[test]
fn only_a_character_set_the_code_does_not_name_is_shown() {
    assert_eq!(row("de_DE.UTF-8", "UTF-8").unnamed_charset(), None);
    assert_eq!(
        row("de_DE", "ISO-8859-1").unnamed_charset(),
        Some("ISO-8859-1")
    );
    assert_eq!(
        row("de_DE@euro", "ISO-8859-15").unnamed_charset(),
        Some("ISO-8859-15")
    );
    assert_eq!(row("sr_RS@latin", "UTF-8").unnamed_charset(), Some("UTF-8"));
}
