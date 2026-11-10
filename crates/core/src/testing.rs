//! Captures other crates' tests read, by the module path that parses them,
//! and the queue's [`Queue`] access for those tests.

pub mod features;
mod queue;

pub use queue::Queue;
