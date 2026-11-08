use std::ops::Range;

use crate::{Terminal, TerminalSize};

pub fn fed(columns: u16, rows: u16, bytes: &[u8]) -> Terminal {
    let mut terminal = Terminal::new(TerminalSize { columns, rows });
    terminal.feed(bytes);
    terminal
}

pub fn texts(terminal: &Terminal) -> Vec<String> {
    terminal.view().row_texts()
}

pub fn numbered_lines(numbers: Range<usize>) -> Vec<u8> {
    numbers
        .flat_map(|number| format!("line{number}\r\n").into_bytes())
        .collect()
}
