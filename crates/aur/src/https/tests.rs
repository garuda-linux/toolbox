use toolbox_https::http::Uri;

use super::HOST;
use crate::DUMP_URL;

#[test]
fn the_pinned_host_is_the_dumps() {
    let url: Uri = DUMP_URL.parse().unwrap();

    assert_eq!(url.host(), Some(HOST));
}
