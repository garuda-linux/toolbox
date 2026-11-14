#[expect(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "fixtures for tests, built from names that always parse"
)]
pub mod kernels;
pub mod packages;
