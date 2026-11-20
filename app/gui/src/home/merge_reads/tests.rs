use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;
use tokio::runtime::Runtime;
use toolbox_core::features::configs::{ConfigEntrySpec, Listing};
use toolbox_exec::ConfigRead;
use toolbox_plan::{ConfigFilePath, Editor, HomePath, MergeCopy};
use toolbox_system::{ConfigFileSource, ReadProblem};

use super::{MergeReads, Merges};
use crate::testing::until;
use crate::waking::Wake;

/// A read that waits behind the password prompt until it is cancelled.
#[derive(Default)]
struct Prompting {
    flags: Mutex<Vec<Arc<AtomicBool>>>,
}

impl Prompting {
    fn flags(&self) -> Vec<Arc<AtomicBool>> {
        self.flags.lock().clone()
    }
}

impl ConfigFileSource for Prompting {
    fn listing(&self, _entries: &[ConfigEntrySpec]) -> Listing {
        Listing::default()
    }

    fn read(
        &self,
        _path: &ConfigFilePath,
        cancel: &Arc<AtomicBool>,
    ) -> Result<ConfigRead, ReadProblem> {
        self.flags.lock().push(Arc::clone(cancel));
        until("the read was never cancelled", || {
            cancel.load(Ordering::SeqCst)
        });
        Err(ReadProblem::Cancelled)
    }

    fn installed(&self, _editor: Editor) -> bool {
        true
    }

    fn home_file(&self, _path: &HomePath) -> Option<PathBuf> {
        None
    }
}

fn copies() -> Vec<MergeCopy> {
    vec![MergeCopy::new("/etc/pacman.conf.pacnew").unwrap()]
}

#[test]
fn closing_the_dialog_cancels_the_read_and_a_new_read_starts_uncancelled() {
    let runtime = Runtime::new().unwrap();
    let source = Arc::new(Prompting::default());
    let mut merges = Merges::default();
    merges.set_reads(MergeReads::new(
        runtime.handle().clone(),
        Wake::none(),
        Arc::<Prompting>::clone(&source),
    ));

    merges.read(copies());
    until("the first read never started", || source.flags().len() == 1);
    merges.stop();

    let first = source.flags().remove(0);
    assert!(first.load(Ordering::SeqCst));

    // The cancelled read still asks for the file in use, at once.
    until("the first read never ended", || source.flags().len() == 2);

    merges.read(copies());
    until("the second read never started", || {
        source.flags().len() == 3
    });

    let second = source.flags().remove(2);
    assert!(!Arc::ptr_eq(&first, &second));
    assert!(!second.load(Ordering::SeqCst));

    merges.stop();
    assert!(second.load(Ordering::SeqCst));
}
