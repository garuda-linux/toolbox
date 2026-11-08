use std::sync::Arc;

use super::Feeds;
use crate::{News, NewsError, NewsSource, Post, Source};

struct Fixed(Result<News, NewsError>);

impl NewsSource for Fixed {
    fn news(&self) -> Result<News, NewsError> {
        self.0.clone()
    }
}

fn post(title: &str, published: Option<i64>, source: Source) -> Post {
    Post {
        title: title.to_owned(),
        link: None,
        published,
        source,
        manual: false,
    }
}

fn feed(posts: Vec<Post>, downloaded: i64) -> Arc<dyn NewsSource> {
    Arc::new(Fixed(Ok(News { posts, downloaded })))
}

fn offline() -> Arc<dyn NewsSource> {
    Arc::new(Fixed(Err(NewsError::Download("no network".to_owned()))))
}

#[test]
fn both_feeds_merge_newest_first() {
    let feeds = Feeds::new(vec![
        feed(
            vec![
                post("Garuda new", Some(300), Source::Garuda),
                post("Garuda old", Some(100), Source::Garuda),
            ],
            10,
        ),
        feed(
            vec![
                post("Arch middle", Some(200), Source::Arch),
                post("Arch undated", None, Source::Arch),
            ],
            20,
        ),
    ]);

    let news = feeds.news().unwrap();
    let titles: Vec<&str> = news.posts.iter().map(|post| post.title.as_str()).collect();

    assert_eq!(
        titles,
        ["Garuda new", "Arch middle", "Garuda old", "Arch undated"]
    );
    assert_eq!(news.downloaded, 20);
}

#[test]
fn one_feed_offline_leaves_the_other() {
    let feeds = Feeds::new(vec![
        offline(),
        feed(vec![post("Arch", Some(1), Source::Arch)], 5),
    ]);

    assert_eq!(feeds.news().unwrap().posts.len(), 1);
}

#[test]
fn every_feed_offline_says_why() {
    let feeds = Feeds::new(vec![offline(), offline()]);

    assert_eq!(
        feeds.news(),
        Err(NewsError::Download("no network".to_owned()))
    );
}
