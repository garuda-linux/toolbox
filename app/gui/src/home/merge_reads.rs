//! Reads for Home's merge dialog, off the frame. Only when the dialog
//! opens, as a root-only file asks for the password.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::runtime::Handle;
use toolbox_gui_kit::Loadable;
use toolbox_plan::MergeCopy;
use toolbox_system::{ConfigFileSource, MergePair, read_pair};

use crate::lane::Lane;
use crate::waking::Wake;

type Landed = Result<Vec<MergePair>, String>;

pub struct MergeReads {
    lane: Lane<Landed>,
    source: Arc<dyn ConfigFileSource>,
}

#[derive(Debug, Default)]
pub struct Merges {
    reads: Option<MergeReads>,
    pairs: Loadable<Vec<MergePair>>,
}

impl std::fmt::Debug for MergeReads {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MergeReads").finish_non_exhaustive()
    }
}

impl MergeReads {
    #[must_use = "nothing is read until the dialog opens"]
    pub fn new(runtime: Handle, wake: Wake, source: Arc<dyn ConfigFileSource>) -> Self {
        Self {
            lane: Lane::new(runtime, wake),
            source,
        }
    }
}

impl Merges {
    pub fn set_reads(&mut self, reads: MergeReads) {
        self.reads = Some(reads);
    }

    pub const fn pairs(&self) -> &Loadable<Vec<MergePair>> {
        &self.pairs
    }

    /// For tests, which read nothing.
    pub fn set_pairs(&mut self, pairs: Vec<MergePair>) {
        self.pairs = Loadable::Loaded(Arc::new(pairs));
    }

    pub fn read(&mut self, copies: Vec<MergeCopy>) {
        let Some(reads) = &self.reads else {
            return;
        };

        self.pairs.start_loading();

        let source = Arc::clone(&reads.source);
        reads.lane.blocking(
            "pairs",
            move || {
                let cancel = Arc::new(AtomicBool::new(false));
                copies
                    .iter()
                    .map(|copy| read_pair(source.as_ref(), copy, &cancel))
                    .collect()
            },
            |result| result.map_err(|error| error.to_string()),
        );
    }

    pub fn take(&mut self) {
        let Some(landed) = self
            .reads
            .as_ref()
            .and_then(|reads| reads.lane.landed().pop())
        else {
            return;
        };

        if let Err(error) = &landed {
            tracing::warn!("the files to merge were not read: {error}");
        }

        self.pairs.finish(landed);
    }
}
