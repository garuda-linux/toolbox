//! Running plan steps: each command in a pseudo-terminal, output streamed
//! as protocol events, answers and cancel taken from the caller. The root
//! helper runs root steps with it, the app runs user steps with it, so both
//! behave the same in the Console.

mod apply;
mod builtin;
mod capture;
mod config_file;
mod feed;
mod initramfs;
mod located;
mod pty_runner;
mod repair;
mod report;
mod sdboot;
mod secure;
#[cfg(test)]
mod testing;

pub use apply::{EventSink, Procedure, Runner, apply, step_span};
pub use builtin::FileChange;
pub use capture::{CaptureError, Captured, Exited, MAX_ERROR_BYTES, Policy, capture, run_captured};
pub use config_file::{
    ConfigRead, Found, MAX_FOLDER_FILES, ReadError, WriteError, inspect, read_config_file,
    refusal_text,
};
pub use located::editor as locate_editor;
pub use pty_runner::{Environment, PtyRunner};
pub use repair::{PreflightError, preflight};
pub use sdboot::sdboot_menu;
