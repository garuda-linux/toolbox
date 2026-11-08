use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener as _};

use super::Outbox;

#[test]
fn replies_accumulate_until_taken() {
    let outbox = Outbox::default();
    outbox.send_event(Event::PtyWrite("\x1b[1;1R".to_owned()));
    outbox.send_event(Event::PtyWrite("\x1b[?6c".to_owned()));

    assert_eq!(outbox.take_replies(), b"\x1b[1;1R\x1b[?6c");
    assert_eq!(outbox.take_replies(), b"");
}

#[test]
fn a_pixel_size_query_is_answered_with_unknown() {
    let outbox = Outbox::default();
    outbox.send_event(Event::TextAreaSizeRequest(Arc::new(|size| {
        format!("{}x{}", size.num_cols, size.cell_width)
    })));

    assert_eq!(outbox.take_replies(), b"0x0");
}
