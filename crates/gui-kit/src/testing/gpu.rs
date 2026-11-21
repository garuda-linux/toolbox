//! A test process may end while lavapipe still runs on its own threads,
//! which then crash in LLVM or corrupt the heap as the exit runs the
//! libraries' destructors (SIGSEGV and SIGABRT after a passed test, seen
//! with Arch's mesa 26.3.1 and LLVM 23). The exit waits for the GPU, then
//! ends the process with its status before those destructors run.

use std::ffi::{c_int, c_void};
use std::sync::Once;

use egui_wgpu::wgpu;
use parking_lot::Mutex;

static DEVICES: Mutex<Vec<wgpu::Device>> = Mutex::new(Vec::new());
static WAITS: Once = Once::new();

#[expect(unsafe_code, reason = "the libc crate binds no on_exit")]
unsafe extern "C" {
    fn on_exit(function: extern "C" fn(c_int, *mut c_void), argument: *mut c_void) -> c_int;
}

/// Makes the process exit wait until `device` finished its work, then
/// end without the libraries' destructors.
pub fn wait_at_exit(device: &wgpu::Device) {
    DEVICES.lock().push(device.clone());

    WAITS.call_once(|| {
        #[expect(unsafe_code, reason = "on_exit has no safe wrapper")]
        // SAFETY: `end` does not unwind and never reads its argument, so a
        // null one is fine. glibc runs it before the libraries' destructors,
        // which it registered earlier.
        unsafe {
            on_exit(end, std::ptr::null_mut());
        }
    });
}

extern "C" fn end(status: c_int, _argument: *mut c_void) {
    for device in DEVICES.lock().iter() {
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .ok();
    }

    // cargo llvm-cov's profile runtime writes its counts in a later exit
    // handler, so a coverage run keeps the whole exit.
    if std::env::var_os("LLVM_PROFILE_FILE").is_some() {
        return;
    }

    #[expect(unsafe_code, reason = "_exit has no safe wrapper")]
    // SAFETY: Rust flushed its standard streams before the exit began, and
    // nothing after this handler needs to run in a test process.
    unsafe {
        libc::_exit(status)
    }
}
