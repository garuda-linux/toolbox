//! The login the add-user dialog proposes while only a full name is typed.

/// shadow-utils' limit for an account name.
const LOGIN_MAX_CHARS: usize = 32;

/// The login the first usable word of `full_name` makes: lowercase ASCII,
/// umlauts and accents folded, `""` when no word makes one. Always a valid
/// user name or empty, never checked against existing accounts.
pub fn login_from_full_name(full_name: &str) -> String {
    full_name
        .split_whitespace()
        .map(login_of_word)
        .find(|login| !login.is_empty())
        .unwrap_or_default()
}

fn login_of_word(word: &str) -> String {
    let mut ascii = String::with_capacity(word.len());
    for letter in word.chars().flat_map(char::to_lowercase) {
        match folded(letter) {
            Some(spelling) => ascii.push_str(spelling),
            None => ascii.push(letter),
        }
    }

    ascii
        .chars()
        .filter(|letter| {
            letter.is_ascii_lowercase() || letter.is_ascii_digit() || "_-".contains(*letter)
        })
        .skip_while(|letter| !(letter.is_ascii_lowercase() || *letter == '_'))
        .take(LOGIN_MAX_CHARS)
        .collect()
}

/// The ASCII spelling of a lowercase letter of the Latin alphabets of
/// Europe, `None` for any other character.
const fn folded(letter: char) -> Option<&'static str> {
    Some(match letter {
        'ä' | 'æ' => "ae",
        'ö' | 'œ' => "oe",
        'ü' => "ue",
        'ß' => "ss",
        'à' | 'á' | 'â' | 'ã' | 'å' | 'ā' | 'ą' => "a",
        'ç' | 'ć' | 'č' => "c",
        'ď' | 'đ' => "d",
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => "e",
        'ğ' => "g",
        'ì' | 'í' | 'î' | 'ï' | 'ī' | 'ı' => "i",
        'ł' => "l",
        'ñ' | 'ń' | 'ň' => "n",
        'ò' | 'ó' | 'ô' | 'õ' | 'ø' | 'ō' | 'ő' => "o",
        'ř' => "r",
        'ś' | 'š' | 'ş' => "s",
        'ť' | 'ţ' => "t",
        'ù' | 'ú' | 'û' | 'ū' | 'ů' | 'ű' => "u",
        'ý' | 'ÿ' => "y",
        'ź' | 'ż' | 'ž' => "z",
        _ => return None,
    })
}
