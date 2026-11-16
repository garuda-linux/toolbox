use super::{Fake, FakeClock, START, Scripted, cache, one};
use crate::AurSource as _;
use crate::transport::Validators;

#[test]
fn the_files_are_private() {
    use std::os::unix::fs::PermissionsExt as _;

    let folder = tempfile::tempdir().unwrap();
    let transport = Fake::new([Scripted::Fresh(one(), Validators::default())]);
    let clock = FakeClock::at(START);
    cache(folder.path(), &transport, &clock).index().unwrap();

    for name in ["index.json", "meta.json"] {
        let mode = std::fs::metadata(folder.path().join(name))
            .unwrap()
            .permissions()
            .mode();

        assert_eq!(mode & 0o777, 0o600, "{name}");
    }
}
