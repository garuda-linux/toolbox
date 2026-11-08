use std::path::Path;

use super::ComponentsPage;
use crate::catalogs::Catalogs;
use crate::testing::{PageFixture, assert_rows_wait_while_busy};

fn shipped() -> PageFixture {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/catalog");
    PageFixture {
        catalogs: Catalogs::load(&dir),
        ..PageFixture::default()
    }
}

#[test]
fn while_a_plan_runs_the_entries_wait_and_a_banner_says_why() {
    assert_rows_wait_while_busy(shipped, ComponentsPage::default);
}
