//! A test process may end while lavapipe still runs the last frame's draw
//! on its own thread, which then crashes in LLVM as the exit unloads the
//! libraries (SIGSEGV and SIGABRT after a passed test). The exit waits for
//! the GPU first.

use std::sync::Once;

use egui_wgpu::wgpu;
use parking_lot::Mutex;

static DEVICES: Mutex<Vec<wgpu::Device>> = Mutex::new(Vec::new());
static WAITS: Once = Once::new();

/// Makes the process exit wait until `device` finished its work.
pub fn wait_at_exit(device: &wgpu::Device) {
    DEVICES.lock().push(device.clone());

    WAITS.call_once(|| {
        #[expect(unsafe_code, reason = "atexit has no safe wrapper")]
        // SAFETY: `wait` takes no arguments and does not unwind. glibc runs
        // it before the libraries' destructors, which it registered earlier.
        unsafe {
            libc::atexit(wait);
        }
    });
}

extern "C" fn wait() {
    for device in DEVICES.lock().iter() {
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .ok();
    }
}
