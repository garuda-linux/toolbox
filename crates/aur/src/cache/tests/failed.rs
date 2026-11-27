use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{DAY, Fake, FakeClock, START, Scripted, cache, one};
use crate::transport::{Answer, Transport, Validators};
use crate::{AurSource as _, Cache, RequestError};

/// Creates a folder where the copy belongs, so its write fails, and then
/// answers with a fresh dump.
struct BlocksCopy(PathBuf);

impl Transport for BlocksCopy {
    fn get(&self, _url: &str, _validators: &Validators) -> Result<Answer, RequestError> {
        std::fs::create_dir_all(self.0.join("index.json")).unwrap();
        Ok(Answer::Fresh {
            body: Box::new(Cursor::new(one())),
            validators: Validators::default(),
        })
    }
}

fn failed_flag(folder: &Path) -> bool {
    let meta = std::fs::read(folder.join("meta.json")).unwrap();
    let meta: serde_json::Value = serde_json::from_slice(&meta).unwrap();

    meta["failed"].as_bool().unwrap()
}

#[test]
fn a_copy_that_cannot_be_written_leaves_the_failed_flag_stored() {
    let folder = tempfile::tempdir().unwrap();
    let clock = FakeClock::at(START);
    let aur = Cache::new(
        folder.path().to_path_buf(),
        Arc::new(BlocksCopy(folder.path().to_path_buf())),
        clock,
    );

    assert!(aur.index().is_ok(), "the fresh dump still answers");
    assert!(failed_flag(folder.path()));
}

#[test]
fn a_copy_stored_after_a_failed_attempt_clears_the_flag() {
    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([
        Scripted::Fails,
        Scripted::Fresh(one(), Validators::default()),
    ]);

    let clock = FakeClock::at(START);

    let aur = cache(folder.path(), &transport, &clock);
    aur.index().unwrap_err();
    assert!(failed_flag(folder.path()));

    clock.set(START + DAY);

    aur.index().unwrap();
    assert!(!failed_flag(folder.path()));
}
