//! Every feed as one list, newest first. A feed without a copy leaves the
//! others shown.

#[cfg(test)]
mod tests;

use std::cmp::Reverse;
use std::path::Path;
use std::sync::Arc;

use crate::{Cache, News, NewsError, NewsSource, Source};

pub struct Feeds {
    feeds: Vec<Arc<dyn NewsSource>>,
}

impl Feeds {
    #[must_use = "the feeds do nothing until asked"]
    pub fn new(feeds: Vec<Arc<dyn NewsSource>>) -> Self {
        Self { feeds }
    }

    #[must_use = "the feeds do nothing until asked"]
    pub fn system(folder: &Path) -> Self {
        Self::new(
            Source::ALL
                .into_iter()
                .map(|source| {
                    let feed: Arc<dyn NewsSource> =
                        Arc::new(Cache::system(folder.to_path_buf(), source));

                    feed
                })
                .collect(),
        )
    }
}

impl NewsSource for Feeds {
    fn news(&self) -> Result<News, NewsError> {
        let mut posts = Vec::new();
        let mut downloaded: Option<i64> = None;
        let mut failure = None;

        for feed in &self.feeds {
            match feed.news() {
                Ok(news) => {
                    posts.extend(news.posts);
                    downloaded = downloaded.max(Some(news.downloaded));
                }
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
        }

        let Some(downloaded) = downloaded else {
            return Err(failure.unwrap_or_else(|| NewsError::Download("no feed".to_owned())));
        };

        // A post without a date sorts last, its place is unknown.
        posts.sort_by_key(|post| Reverse(post.published));

        Ok(News { posts, downloaded })
    }
}
