mod store;

use std::fs;
use std::os::unix::fs::{PermissionsExt as _, symlink};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use tempfile::TempDir;
use toolbox_plan::protocol::Outcome;
use toolbox_plan::{
    AUR_BUILT, AUR_STORE, BuildRun, Caller, InstallReason, PackageName, TargetList,
};

use super::{Install, StageError, home_of, install, package_of, stage, unstage};

/// The home of `user` in the passwd capture.
const HOME: &str = "home/user.guest";
const OCTOPI: &str = "octopi-git-0.16.2.r31.d98b3541-1-x86_64.pkg.tar.zst";

fn uid() -> u32 {
    rustix::process::geteuid().as_raw()
}

fn run() -> BuildRun {
    BuildRun::new("1a2b").unwrap()
}

fn packages(names: &[&str]) -> TargetList<PackageName> {
    TargetList::new(names.iter().map(|name| PackageName::new(*name).unwrap())).unwrap()
}

/// A root with the builder's home and an empty run folder, each 0755.
fn system() -> (TempDir, PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let folder = root.path().join(HOME).join(AUR_BUILT).join("1a2b");
    fs::create_dir_all(&folder).unwrap();

    for dir in folder.ancestors().take_while(|dir| *dir != root.path()) {
        fs::set_permissions(dir, fs::Permissions::from_mode(0o755)).unwrap();
    }

    (root, folder)
}

fn staged(root: &Path, wanted: &[&str]) -> Result<Vec<PathBuf>, StageError> {
    stage(root, uid(), Path::new(HOME), &run(), &packages(wanted))
}

#[test]
fn file_names_give_their_package() {
    assert_eq!(package_of(OCTOPI), Some("octopi-git"));
    assert_eq!(
        package_of("nvidia-470xx-utils-470.256.02-8.03-x86_64.pkg.tar.zst"),
        Some("nvidia-470xx-utils")
    );
    assert_eq!(
        package_of("python-x-1:2.0-1-any.pkg.tar.xz"),
        Some("python-x")
    );
    assert_eq!(
        package_of("octopi-git-debug-0.16-1-x86_64.pkg.tar.zst"),
        Some("octopi-git-debug")
    );
    assert_eq!(package_of("PKGBUILD"), None);
    assert_eq!(package_of("x-1-x86_64.pkg.tar.zst"), None);
}

/// makepkg signs with `--sign` beside the package. The signature is no
/// package, else a signed build would hold two files of one package.
#[test]
fn a_signature_is_no_package() {
    assert_eq!(package_of(&format!("{OCTOPI}.sig")), None);
}

#[test]
fn an_uncompressed_package_counts() {
    assert_eq!(
        package_of("octopi-git-0.16-1-x86_64.pkg.tar"),
        Some("octopi-git")
    );
}

#[test]
fn every_compression_makepkg_knows_counts() {
    for compression in [".zst", ".xz", ".gz", ".bz2", ".lz4", ".lzo", ".lz", ".Z"] {
        assert_eq!(
            package_of(&format!("octopi-git-0.16-1-x86_64.pkg.tar{compression}")),
            Some("octopi-git"),
            "{compression}"
        );
    }
}

#[test]
fn an_unknown_ending_is_no_package() {
    assert_eq!(
        package_of("octopi-git-0.16-1-x86_64.pkg.tar.zst.part"),
        None
    );
    assert_eq!(package_of("octopi-git-0.16-1-x86_64.pkg.tar.7z"), None);
    assert_eq!(package_of("octopi-git-0.16-1-x86_64.pkg.tarx"), None);
}

#[test]
fn a_signed_build_stages_its_package() {
    let (root, folder) = system();
    fs::write(folder.join(OCTOPI), b"package").unwrap();
    fs::write(folder.join(format!("{OCTOPI}.sig")), b"signature").unwrap();

    let copies = staged(root.path(), &["octopi-git"]).unwrap();

    assert_eq!(
        copies,
        [Path::new("/").join(AUR_STORE).join("1a2b").join(OCTOPI)]
    );
}

/// Only the asked package is copied: the `-debug` package and other files
/// makepkg left stay behind.
#[test]
fn the_asked_package_is_copied_into_the_store() {
    let (root, folder) = system();
    fs::write(folder.join(OCTOPI), b"package").unwrap();
    fs::write(
        folder.join("octopi-git-debug-0.16.2.r31.d98b3541-1-x86_64.pkg.tar.zst"),
        b"",
    )
    .unwrap();
    fs::write(folder.join("notes.txt"), b"").unwrap();

    let copies = staged(root.path(), &["octopi-git"]).unwrap();

    let store = Path::new("/").join(AUR_STORE).join("1a2b");
    assert_eq!(copies, [store.join(OCTOPI)]);

    let copy = root.path().join(AUR_STORE).join("1a2b").join(OCTOPI);
    assert_eq!(fs::read(&copy).unwrap(), b"package");
    assert_eq!(
        fs::metadata(&copy).unwrap().permissions().mode() & 0o777,
        0o644
    );
}

#[test]
fn a_package_without_a_file_fails() {
    let (root, _folder) = system();

    assert!(matches!(
        staged(root.path(), &["octopi-git"]),
        Err(StageError::NotBuilt(name)) if name.as_str() == "octopi-git"
    ));
}

/// Two versions in one run would leave the choice to chance.
#[test]
fn two_files_of_one_package_fail() {
    let (root, folder) = system();
    fs::write(folder.join(OCTOPI), b"").unwrap();
    fs::write(folder.join("octopi-git-0.16.3-1-x86_64.pkg.tar.zst"), b"").unwrap();

    assert!(matches!(
        staged(root.path(), &["octopi-git"]),
        Err(StageError::Ambiguous(_))
    ));
}

/// Another user of that group could swap the file after the check.
#[test]
fn a_folder_others_may_write_fails() {
    let (root, folder) = system();
    fs::write(folder.join(OCTOPI), b"").unwrap();
    fs::set_permissions(&folder, fs::Permissions::from_mode(0o775)).unwrap();

    assert!(matches!(
        staged(root.path(), &["octopi-git"]),
        Err(StageError::Shared(_))
    ));
}

#[test]
fn a_file_others_may_write_fails() {
    let (root, folder) = system();
    fs::write(folder.join(OCTOPI), b"").unwrap();
    fs::set_permissions(folder.join(OCTOPI), fs::Permissions::from_mode(0o666)).unwrap();

    assert!(matches!(
        staged(root.path(), &["octopi-git"]),
        Err(StageError::Shared(_))
    ));
}

/// A link could point root at any file on the system.
#[test]
fn a_linked_folder_fails() {
    let (root, folder) = system();
    let elsewhere = root.path().join("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    fs::write(elsewhere.join(OCTOPI), b"").unwrap();
    fs::remove_dir(&folder).unwrap();
    symlink(&elsewhere, &folder).unwrap();

    assert!(matches!(
        staged(root.path(), &["octopi-git"]),
        Err(StageError::Io(_))
    ));
    assert!(!root.path().join(AUR_STORE).exists());
}

#[test]
fn a_folder_of_another_user_fails() {
    let (root, folder) = system();
    fs::write(folder.join(OCTOPI), b"").unwrap();

    assert!(matches!(
        stage(
            root.path(),
            uid() + 1,
            Path::new(HOME),
            &run(),
            &packages(&["octopi-git"])
        ),
        Err(StageError::Foreign(_))
    ));
}

#[test]
fn copies_are_removed_after_the_install() {
    let (root, folder) = system();
    fs::write(folder.join(OCTOPI), b"").unwrap();
    staged(root.path(), &["octopi-git"]).unwrap();

    unstage(root.path(), &run()).unwrap();

    assert!(!root.path().join(AUR_STORE).join("1a2b").exists());
}

#[test]
fn the_home_of_the_caller_comes_from_passwd() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join("etc")).unwrap();
    fs::write(
        root.path().join("etc/passwd"),
        toolbox_plan::testing::accounts::PASSWD,
    )
    .unwrap();

    assert_eq!(
        home_of(root.path(), 1000).unwrap(),
        PathBuf::from("home/user.guest")
    );
    assert!(matches!(
        home_of(root.path(), 4242),
        Err(StageError::NoAccount(4242))
    ));
}

/// The passwd capture with its `user` given the uid of the test.
fn passwd_with_caller(root: &Path) {
    fs::create_dir_all(root.join("etc")).unwrap();
    let passwd = toolbox_plan::testing::accounts::PASSWD
        .replace("user:x:1000:1000:", &format!("user:x:{}:1000:", uid()));

    fs::write(root.join("etc/passwd"), passwd).unwrap();
}

fn octopi(reason: InstallReason) -> Install<'static> {
    static RUN: LazyLock<BuildRun> = LazyLock::new(run);
    static PACKAGES: LazyLock<TargetList<PackageName>> =
        LazyLock::new(|| packages(&["octopi-git"]));

    Install {
        run: &RUN,
        packages: &PACKAGES,
        reason,
    }
}

/// The root the install ran in and the commands pacman got.
fn installed_with(reason: InstallReason) -> (TempDir, Vec<String>) {
    let (root, folder) = system();
    passwd_with_caller(root.path());
    fs::write(folder.join(OCTOPI), b"").unwrap();

    let mut ran = Vec::new();
    let outcome = install(root.path(), Caller::new(uid()), octopi(reason), |command| {
        ran.push(command.to_string());
        Outcome::Succeeded
    });

    assert_eq!(outcome.unwrap(), Outcome::Succeeded);
    (root, ran)
}

#[test]
fn the_copies_are_installed_with_pacman_and_removed() {
    let (root, ran) = installed_with(InstallReason::Explicit);

    assert_eq!(
        ran,
        [format!(
            "pacman -U --asexplicit --noconfirm -- /{AUR_STORE}/1a2b/{OCTOPI}"
        )]
    );
    assert!(!root.path().join(AUR_STORE).join("1a2b").exists());
}

#[test]
fn a_dependency_is_installed_as_one() {
    let (_root, ran) = installed_with(InstallReason::Dependency);

    assert_eq!(
        ran,
        [format!(
            "pacman -U --asdeps --noconfirm -- /{AUR_STORE}/1a2b/{OCTOPI}"
        )]
    );
}

/// An upgrade of an installed package keeps the reason pacman has for it.
#[test]
fn a_kept_reason_passes_no_reason_flag() {
    let (_root, ran) = installed_with(InstallReason::Keep);

    assert_eq!(
        ran,
        [format!(
            "pacman -U --noconfirm -- /{AUR_STORE}/1a2b/{OCTOPI}"
        )]
    );
}

/// Without a caller the helper does not know whose home to read.
#[test]
fn no_caller_installs_nothing() {
    let (root, _folder) = system();

    let outcome = install(
        root.path(),
        None,
        octopi(InstallReason::Dependency),
        |_command| panic!("pacman ran without a caller"),
    );

    assert!(matches!(outcome, Err(StageError::NoCaller)));
}
