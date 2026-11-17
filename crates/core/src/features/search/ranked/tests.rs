use super::{Rank, Ranked, by_best, top};
use crate::features::search::Needle;

#[test]
fn a_name_ranks_exact_then_prefix_then_inside_then_by_its_other_texts() {
    let needle = Needle::new("Fire");

    assert_eq!(needle.rank("firefox", ["Web browser"]), Some(Rank::Prefix));
    assert_eq!(needle.rank("FIRE", []), Some(Rank::Exact));
    assert_eq!(needle.rank("campfire", []), Some(Rank::InName));
    assert_eq!(
        needle.rank("ufw", ["A simple firewall"]),
        Some(Rank::InText)
    );
    assert_eq!(needle.rank("ufw", ["Uncomplicated"]), None);
}

#[test]
fn an_empty_search_ranks_nothing() {
    assert_eq!(Needle::new("  ").rank("firefox", []), None);
}

#[test]
fn folded_texts_rank_the_same_way() {
    let needle = Needle::new("Fire");

    assert_eq!(needle.rank_folded("fire", ""), Some(Rank::Exact));
    assert_eq!(needle.rank_folded("firefox", ""), Some(Rank::Prefix));
    assert_eq!(needle.rank_folded("campfire", ""), Some(Rank::InName));
    assert_eq!(
        needle.rank_folded("ufw", "a simple firewall"),
        Some(Rank::InText)
    );
    assert_eq!(needle.rank_folded("ufw", "uncomplicated"), None);
}

#[test]
fn a_section_keeps_its_best_hits_in_rank_then_given_order_and_counts_all() {
    let hits = [
        (Rank::InText, "d"),
        (Rank::Prefix, "b"),
        (Rank::Exact, "a"),
        (Rank::Prefix, "c"),
    ];

    let ranked = top(hits, 2).expect("hits");

    assert_eq!(ranked.best, Rank::Exact);
    assert_eq!(ranked.hits, ["a", "b"]);
    assert_eq!(ranked.total, 4);
    assert!(top(Vec::<(Rank, &str)>::new(), 2).is_none());
}

#[test]
fn sections_go_by_their_best_hit_then_by_their_given_order() {
    let section = |best| Ranked {
        best,
        hits: Vec::<()>::new(),
        total: 1,
    };

    let mut sections = vec![
        ("pages", section(Rank::InName)),
        ("packages", section(Rank::Exact)),
        ("apps", section(Rank::InName)),
    ];

    by_best(&mut sections, |(_, ranked)| ranked.best);

    let order: Vec<&str> = sections.iter().map(|(name, _)| *name).collect();
    assert_eq!(order, ["packages", "pages", "apps"]);
}

#[test]
fn a_mapped_section_keeps_its_rank_and_count() {
    let ranked = top([(Rank::Prefix, 1_u32), (Rank::InName, 2_u32)], 1).expect("hits");
    let shown = ranked.map(|hit| hit * 10);

    assert_eq!(
        shown,
        Ranked {
            best: Rank::Prefix,
            hits: vec![10],
            total: 2,
        }
    );
}
