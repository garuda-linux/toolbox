//! Request lines written by hand, for what a typed request cannot hold.

use toolbox_plan::protocol::PROTOCOL_VERSION;

pub fn request_line(mode: &str, action: &str) -> String {
    format!(
        "{{\"version\":{PROTOCOL_VERSION},\"mode\":\"{mode}\",\"steps\":[{{\"id\":1,\"action\":{action}}}]}}\n"
    )
}
