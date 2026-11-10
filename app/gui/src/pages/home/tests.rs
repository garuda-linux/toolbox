mod lost;

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable as _;
use toolbox_core::domains::Packages;
use toolbox_core::testing::Queue as _;
use toolbox_core::{ChangeSet, Toggle};
use toolbox_gui_kit::Loadable;
use toolbox_gui_kit::fonts::{self, SMALL, Weight};
use toolbox_news::{News, Post, Source};
use toolbox_plan::PackageName;
use toolbox_system::{DiskSpace, Facts, PackageCount, Status};

use super::HomePage;
use crate::command::{Command, UiCommand};
use crate::page::PageId;
use crate::testing::{PageFixture, PageState};

const GB: u64 = 1_000_000_000;

type Shown = Harness<'static, PageState<HomePage>>;

fn page(fixture: PageFixture) -> Shown {
    let mut harness = fixture.harness(HomePage::default());
    harness.run();
    harness
}

fn link(harness: &Shown, name: &str) -> bool {
    harness.query_by_role_and_label(Role::Link, name).is_some()
}

#[test]
fn the_setup_assistant_is_a_step_until_it_ran() {
    let harness = page(PageFixture::default());
    assert!(link(&harness, "Setup Assistant"));
    assert!(link(&harness, "Maintenance"));
    assert!(link(&harness, "Diagnostics"));

    let finished = page(PageFixture {
        setup_finished: true,
        ..PageFixture::default()
    });

    assert!(!link(&finished, "Setup Assistant"));
    assert!(link(&finished, "Diagnostics"));
}

#[test]
fn queued_changes_come_first_and_push_out_the_last_step() {
    let mut changes = ChangeSet::default();
    changes.want::<Packages>(
        PackageName::new("strawberry").unwrap(),
        Toggle::On,
        &toolbox_core::Snapshot::default(),
    );
    let harness = page(PageFixture {
        changes,
        ..PageFixture::default()
    });

    assert!(link(&harness, "Review changes"));
    assert!(link(&harness, "Setup Assistant"));
    assert!(!link(&harness, "Diagnostics"), "only three steps fit");
}

#[test]
fn the_pages_used_last_open_from_their_chips() {
    let mut harness = page(PageFixture {
        recent: vec![PageId::Network, PageId::Boot],
        ..PageFixture::default()
    });

    assert!(harness.query_by_label("Recently used").is_some());
    harness
        .get_by_role_and_label(Role::Button, "Network")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(UiCommand::Navigate(PageId::Network))]
    );
}

#[test]
fn without_pages_used_last_there_is_no_row() {
    let harness = page(PageFixture::default());

    assert!(harness.query_by_label("Recently used").is_none());
}

fn news(posts: Vec<Post>) -> Loadable<News> {
    Loadable::Loaded(Arc::new(News {
        posts,
        downloaded: 0,
    }))
}

fn post(title: &str, link: Option<&str>) -> Post {
    Post {
        title: title.to_owned(),
        link: link.map(str::to_owned),
        published: Some(1_792_327_265),
        source: Source::Garuda,
        manual: false,
    }
}

#[test]
fn the_news_lists_the_newest_posts_with_their_day() {
    let posts = (1_u8..=5)
        .map(|number| {
            post(
                &format!("Post {number}"),
                Some("https://forum.garudalinux.org/t/1"),
            )
        })
        .collect();

    let harness = page(PageFixture {
        news: Some(news(posts)),
        ..PageFixture::default()
    });

    assert!(harness.query_by_label("News").is_some());
    assert!(link(&harness, "Open Post 1"));
    assert!(link(&harness, "Open Post 4"));
    assert!(!link(&harness, "Open Post 5"), "four posts only");
    assert!(harness.query_all_by_label("2026-10-18").next().is_some());
}

#[test]
fn a_post_without_a_forum_link_is_plain_text() {
    let harness = page(PageFixture {
        news: Some(news(vec![post("Elsewhere", Some("https://example.org/x"))])),
        ..PageFixture::default()
    });

    assert!(harness.query_by_label("Elsewhere").is_some());
    assert!(!link(&harness, "Open Elsewhere"));
}

#[test]
fn without_posts_there_is_no_news_section() {
    let harness = page(PageFixture {
        news: Some(news(Vec::new())),
        ..PageFixture::default()
    });

    assert!(harness.query_by_label("News").is_none());
}

#[test]
fn an_arch_post_names_its_source_and_its_manual_step() {
    let arch = Post {
        title: "glibc needs a manual intervention".to_owned(),
        link: Some("https://archlinux.org/news/glibc/".to_owned()),
        published: Some(1_792_327_265),
        source: Source::Arch,
        manual: true,
    };

    let harness = page(PageFixture {
        news: Some(news(vec![arch, post("Garuda post", None)])),
        ..PageFixture::default()
    });

    assert!(link(&harness, "Open glibc needs a manual intervention"));
    assert!(harness.query_by_label("Arch").is_some());
    assert!(harness.query_by_label("Garuda").is_some());
    assert!(harness.query_by_label("Manual step").is_some());
    assert!(link(&harness, "Arch news"));
    assert!(link(&harness, "Garuda forum"));
}

#[test]
fn an_arch_post_cannot_link_to_the_forum() {
    let wrong = Post {
        title: "Odd".to_owned(),
        link: Some("https://forum.garudalinux.org/t/1".to_owned()),
        published: None,
        source: Source::Arch,
        manual: false,
    };

    let harness = page(PageFixture {
        news: Some(news(vec![wrong])),
        ..PageFixture::default()
    });

    assert!(!link(&harness, "Open Odd"));
    assert!(harness.query_by_label("Odd").is_some());
}

fn facts() -> Facts {
    Facts {
        packages: Ok(PackageCount {
            installed: 1_462,
            foreign: Some(12),
        }),
        kernel: Ok("6.17.7-zen1-1-zen".to_owned()),
        booted: Ok(SystemTime::now() - Duration::from_hours(100)),
        disk: Ok(DiskSpace {
            total: 500 * GB,
            free: 200 * GB,
        }),
        package_cache: Ok(0),
        orphans: Ok(Vec::new()),
        failed_units: Ok(Vec::new()),
    }
}

/// Updated 16 days ago, so the summary has a second part.
fn status() -> Status {
    Status {
        updates: Ok(Vec::new()),
        aur_updates: Ok(Vec::new()),
        merges: Ok(Vec::new()),
        last_update: Ok(Some(SystemTime::now() - Duration::from_hours(16 * 24 + 1))),
        health: Ok(Vec::new()),
    }
}

fn greeted() -> Shown {
    page(PageFixture {
        status: Some(Loadable::Loaded(Arc::new(status()))),
        facts: Some(Loadable::Loaded(Arc::new(facts()))),
        ..PageFixture::default()
    })
}

#[test]
fn the_package_count_is_a_tag_like_the_other_facts_and_opens_packages() {
    let mut harness = greeted();
    let packages = harness
        .get_by_role_and_label(Role::Link, "1462 packages · 12 AUR")
        .rect();

    let kernel = harness.get_by_label_contains("6.17.7-zen1-1-zen").rect();

    assert!((packages.height() - kernel.height()).abs() < 0.5);
    assert!((packages.center().y - kernel.center().y).abs() < 0.5);

    harness
        .get_by_role_and_label(Role::Link, "1462 packages · 12 AUR")
        .click();
    harness.run();

    assert_eq!(
        harness.state().fixture.sent(),
        [Command::from(UiCommand::Navigate(PageId::Packages))]
    );
}

#[test]
fn the_summary_dot_has_a_space_on_both_sides() {
    let harness = greeted();
    let attention = harness.get_by_label("One thing needs attention").rect();
    let updated = harness.get_by_label("· last update 16 days ago").rect();

    let font = fonts::font(&harness.ctx.global_style(), Weight::Regular, SMALL);
    let space = harness.ctx.fonts_mut(|fonts| fonts.glyph_width(&font, ' '));

    let gap = updated.left() - attention.right();
    assert!(
        (gap - space).abs() < 1.0,
        "{gap} before the dot, {space} after it"
    );
}

#[test]
fn the_summary_status_and_last_update_share_the_small_size() {
    let harness = greeted();
    let size = |text: &str| {
        let shape = toolbox_gui_kit::testing::painted_text(&harness, text)?;
        shape
            .galley
            .job
            .sections
            .first()
            .map(|part| part.format.font_id.size)
    };

    let small = fonts::font(&harness.ctx.global_style(), Weight::Regular, SMALL).size;
    assert_eq!(size("One thing needs attention"), Some(small));
    assert_eq!(size("· last update 16 days ago"), Some(small));
}
