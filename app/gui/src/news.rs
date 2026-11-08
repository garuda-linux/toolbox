//! The fixed posts the dry run of a self-test build shows instead of
//! asking Garuda's forum and Arch's website. Release builds leave them out.

use toolbox_news::{News, NewsError, NewsSource, Post, Source};

/// 2025-01-10, so dates on the page read the same on every run.
const DAY: i64 = 1_736_467_200;
const SECONDS_PER_DAY: i64 = 86_400;

/// The news of a dry run: fixed posts of both sources, one of them asking
/// for a manual step, so the self-test and a look at the page never reach
/// the network.
pub struct SampleNews;

impl NewsSource for SampleNews {
    fn news(&self) -> Result<News, NewsError> {
        let posts = [
            (
                "Toolbox sample: a new kernel series",
                0,
                Source::Garuda,
                false,
            ),
            ("Toolbox sample: manual intervention", 3, Source::Arch, true),
            (
                "Toolbox sample: the keyring changed",
                7,
                Source::Garuda,
                false,
            ),
            ("Toolbox sample: a mirror moved", 12, Source::Arch, false),
            (
                "Toolbox sample: monthly ISO release",
                20,
                Source::Garuda,
                false,
            ),
        ]
        .into_iter()
        .map(|(title, days_ago, source, manual)| Post {
            title: title.to_owned(),
            link: Some(source.page().to_owned()),
            published: Some(DAY - days_ago * SECONDS_PER_DAY),
            source,
            manual,
        })
        .collect();

        Ok(News {
            posts,
            downloaded: DAY,
        })
    }
}
