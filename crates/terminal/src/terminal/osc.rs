//! Follows an operating system command string (`ESC ]` to BEL or ST)
//! across the calls of `feed`. The parser keeps such a string without a
//! limit, so a program that never ends one would grow the memory without
//! bound. The parser reads UTF-8, so the 8-bit C1 bytes 0x9c and 0x9d
//! neither start nor end a string there, and neither do they here.

const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;
const SUB: u8 = 0x1a;

/// The byte that aborts a string in the parser. The parser drops the
/// string and goes on with the text after it.
pub const CAN: u8 = 0x18;

/// Longest string the parser may keep. Longer ones are cut off.
pub const MAX_OSC_BYTES: usize = 64 << 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Ground,
    Escape,
    Str,
}

/// What a byte does to the string the parser holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Kept,
    /// The string is past [`MAX_OSC_BYTES`]: the byte belongs to it, and
    /// [`CAN`] has to follow it in the parser.
    Overlong,
}

#[derive(Debug, Default)]
pub struct Tracker {
    state: State,
    bytes: usize,
}

impl Tracker {
    pub fn step(&mut self, byte: u8) -> Step {
        let (state, step) = match (self.state, byte) {
            (State::Str, BEL | CAN | SUB) => (State::Ground, Step::Kept),
            (State::Str, ESC) => (State::Escape, Step::Kept),
            (State::Str, _) if self.bytes >= MAX_OSC_BYTES => (State::Ground, Step::Overlong),
            (State::Str, _) => {
                self.bytes += 1;
                (State::Str, Step::Kept)
            }
            (_, ESC) => (State::Escape, Step::Kept),
            (State::Escape, b']') => (State::Str, Step::Kept),
            _ => (State::Ground, Step::Kept),
        };

        if state != State::Str {
            self.bytes = 0;
        }

        self.state = state;
        step
    }
}
