use super::StderrLogger;

fn metadata(level: log::Level) -> log::Metadata<'static> {
    log::Metadata::builder().level(level).target("test").build()
}

#[test]
fn info_and_below_is_on() {
    let logger = StderrLogger;

    assert!(log::Log::enabled(&logger, &metadata(log::Level::Error)));
    assert!(log::Log::enabled(&logger, &metadata(log::Level::Info)));
    assert!(!log::Log::enabled(&logger, &metadata(log::Level::Debug)));
}
