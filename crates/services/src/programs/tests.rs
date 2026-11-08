use std::path::Path;

use super::Programs;

#[test]
fn each_program_is_looked_up_at_its_own_path() {
    let only_chroot = Programs::found(|path| path == Path::new("/usr/bin/garuda-chroot"));

    assert_eq!((only_chroot.installer, only_chroot.chroot), (false, true));

    let all = Programs::found(|_| true);
    assert_eq!((all.installer, all.chroot), (true, true));
}

#[test]
fn maintenance_sees_its_tools_and_repairs() {
    let found = Programs::found(|path| {
        path == Path::new("/usr/bin/garuda-update") || path == Path::new("/usr/bin/pace")
    });

    assert!(found.garuda_update);
    let installed: Vec<&str> = found
        .tools
        .iter()
        .filter(|row| row.installed)
        .map(|row| row.tool)
        .collect();

    assert_eq!(installed, ["edit-repositories"]);

    let available: Vec<&str> = found
        .repairs
        .iter()
        .filter(|row| row.available)
        .map(|row| row.repair.as_str())
        .collect();

    assert_ne!(available, Vec::<&str>::new());
    assert!(
        available.iter().all(|repair| repair.starts_with("remote-")),
        "{available:?}"
    );
}
