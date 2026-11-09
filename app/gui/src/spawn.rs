//! Background tasks whose end nobody awaits. tokio keeps a task's panic in
//! its `JoinHandle`, so a dropped handle would end the task without a word
//! of which one it was.

use tokio::runtime::Handle;

/// Runs `future` on `runtime` and logs which task it was when it panics.
/// The panic hook logs the message and the place.
pub fn spawn_logged<F>(runtime: &Handle, what: &'static str, future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let task = runtime.spawn(future);

    drop(runtime.spawn(async move {
        if let Err(error) = task.await
            && error.is_panic()
        {
            tracing::warn!("the {what} task ended by a panic");
        }
    }));
}
