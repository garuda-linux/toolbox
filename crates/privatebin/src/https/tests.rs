use toolbox_https::http::Uri;

use super::HOST;
use crate::client::SERVER;

#[test]
fn the_pinned_host_is_the_servers() {
    let url: Uri = SERVER.parse().unwrap();

    assert_eq!(url.host(), Some(HOST));
}
