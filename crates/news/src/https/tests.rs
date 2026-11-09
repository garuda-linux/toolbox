use toolbox_https::http::Uri;

use crate::Source;

#[test]
fn each_pinned_host_is_its_feeds() {
    for source in Source::ALL {
        let url: Uri = source.feed().parse().unwrap();

        assert_eq!(url.host(), Some(source.host()), "{source:?}");
    }
}
