use std::fs;
use std::path::{Path, PathBuf};

use super::{parse, read, wildcard_match};

/// `/etc/pacman.conf` of the Lima VM. Its Garuda and Chaotic-AUR
/// sections include `/etc/pacman.d/chaotic-mirrorlist`.
const LIMA: &str = include_str!("fixtures/pacman.conf");
const MIRRORLIST: &str = include_str!("fixtures/etc/pacman.d/mirrorlist");
const CHAOTIC_MIRRORLIST: &str = include_str!("fixtures/etc/pacman.d/chaotic-mirrorlist");

const LIMA_REPOS: [&str; 5] = ["garuda", "core", "extra", "multilib", "chaotic-aur"];

/// The Lima conf with each `(from, to)` replaced once. Every `from` must be
/// in it, so a changed capture cannot pass a test unedited.
fn lima_with(edits: &[(&str, &str)]) -> String {
    edits.iter().fold(LIMA.to_owned(), |text, (from, to)| {
        assert!(text.contains(from), "{from:?} is not in the Lima conf");
        text.replacen(from, to, 1)
    })
}

fn repos(conf: &super::PacmanConf) -> Vec<&str> {
    conf.repos.iter().map(String::as_str).collect()
}

fn write(folder: &Path, name: &str, text: impl AsRef<[u8]>) -> PathBuf {
    let path = folder.join(name);

    fs::write(&path, text).unwrap();
    path
}

/// The Lima conf and its mirrorlists in `folder`, its includes pointed
/// there.
fn lima_in(folder: &Path, conf: &str) -> PathBuf {
    let pacman_d = folder.join("pacman.d");

    fs::create_dir_all(&pacman_d).unwrap();
    write(&pacman_d, "mirrorlist", MIRRORLIST);
    write(&pacman_d, "chaotic-mirrorlist", CHAOTIC_MIRRORLIST);
    write(
        folder,
        "pacman.conf",
        conf.replace("/etc/pacman.d/", &format!("{}/", pacman_d.display())),
    )
}

#[test]
fn the_lima_conf_has_its_repositories_in_file_order_and_the_defaults() {
    let conf = parse(LIMA);

    assert_eq!(repos(&conf), LIMA_REPOS);
    assert_eq!(conf.db_path, PathBuf::from("/var/lib/pacman"));
    assert_eq!(conf.root, PathBuf::from("/"));
}

/// The Lima conf with its `DBPath` line uncommented and comments added
/// behind it and behind `[core]`.
#[test]
fn trailing_comments_are_cut_like_libalpm_cuts_them() {
    let conf = parse(&lima_with(&[
        (
            "#DBPath      = /var/lib/pacman/",
            "DBPath      = /var/lib/pacman/   # where the databases live",
        ),
        ("[core]\n", "[core] # the base system\n"),
    ]));

    assert_eq!(repos(&conf), LIMA_REPOS);
    assert_eq!(conf.db_path, PathBuf::from("/var/lib/pacman/"));
    assert_eq!(conf.root, PathBuf::from("/"));
}

/// The Lima conf with `RootDir = /mnt#chroot` and `[garuda]#ours`.
#[test]
fn a_value_with_a_comment_keeps_only_what_comes_before_it() {
    let conf = parse(&lima_with(&[
        ("#RootDir     = /", "RootDir = /mnt#chroot"),
        ("[garuda]\n", "[garuda]#ours\n"),
    ]));

    assert_eq!(conf.root, PathBuf::from("/mnt"));
    assert_eq!(repos(&conf), LIMA_REPOS);
}

/// Garuda's `pacman.conf` may keep its own repositories in an included
/// file, which pacman accepts. Here the `[garuda]` section of the Lima
/// conf is moved into one.
#[test]
fn repositories_in_an_included_file_count_in_their_place() {
    let folder = tempfile::tempdir().unwrap();
    let section = "[garuda]\nInclude = /etc/pacman.d/chaotic-mirrorlist\n";
    let garuda = write(folder.path(), "garuda.conf", section);
    let main = lima_in(
        folder.path(),
        &lima_with(&[(section, &format!("Include = {}\n", garuda.display()))]),
    );

    assert_eq!(repos(&read(&main).unwrap()), LIMA_REPOS);
}

/// The Lima conf with a wildcard include appended. Garuda ships no
/// drop-in folder, so its files are made up.
#[test]
fn wildcard_includes_are_read_in_sorted_order() {
    let folder = tempfile::tempdir().unwrap();
    let drop_in = folder.path().join("repos.d");

    fs::create_dir_all(&drop_in).unwrap();
    write(&drop_in, "20-second.conf", "[second]\n");
    write(&drop_in, "10-first.conf", "[first]\n");
    write(&drop_in, ".hidden.conf", "[hidden]\n");
    write(&drop_in, "notes.txt", "[notes]\n");

    let main = lima_in(
        folder.path(),
        &format!("{LIMA}Include = {}/*.conf\n", drop_in.display()),
    );

    assert_eq!(
        repos(&read(&main).unwrap()),
        [LIMA_REPOS.as_slice(), &["first", "second"]].concat()
    );
}

/// The Lima conf with its mirrorlists gone and an include of itself
/// appended.
#[test]
fn a_missing_include_or_one_that_includes_itself_ends() {
    let folder = tempfile::tempdir().unwrap();
    let main = folder.path().join("pacman.conf");

    fs::write(
        &main,
        format!(
            "{}Include = {}\n",
            LIMA.replace("/etc/pacman.d/", "/nonexistent/pacman.d/"),
            main.display()
        ),
    )
    .unwrap();

    assert_eq!(repos(&read(&main).unwrap()), LIMA_REPOS);
}

/// The Lima conf with a Latin-1 comment put in front.
#[test]
fn a_stray_byte_hides_no_repository() {
    let folder = tempfile::tempdir().unwrap();
    let main = lima_in(folder.path(), LIMA);
    let mut text = b"# caf\xe9\n".to_vec();
    text.extend_from_slice(fs::read(&main).unwrap().as_slice());

    fs::write(&main, text).unwrap();

    assert_eq!(repos(&read(&main).unwrap()), LIMA_REPOS);
}

#[test]
fn the_lima_conf_is_read_with_its_mirrorlists() {
    let folder = tempfile::tempdir().unwrap();

    assert_eq!(
        repos(&read(&lima_in(folder.path(), LIMA)).unwrap()),
        LIMA_REPOS
    );
}

#[test]
fn a_missing_pacman_conf_is_an_error() {
    read(Path::new("/nonexistent/pacman.conf")).unwrap_err();
}

#[test]
fn wildcards_match_like_glob() {
    for (pattern, name, matches) in [
        ("*.conf", "a.conf", true),
        ("*.conf", "a.conf.bak", false),
        ("*.conf", ".a.conf", false),
        ("?.conf", "a.conf", true),
        ("?.conf", "ab.conf", false),
        ("a*b*c", "aXbYc", true),
        ("a*b*c", "aXbY", false),
        ("*", "anything", true),
    ] {
        assert_eq!(wildcard_match(pattern, name), matches, "{pattern} {name}");
    }
}

#[test]
fn an_empty_config_has_the_default_databases_and_no_repos() {
    let conf = parse("");

    assert_eq!(conf.root, PathBuf::from("/"));
    assert_eq!(conf.db_path, PathBuf::from("/var/lib/pacman"));
    assert_eq!(repos(&conf), Vec::<&str>::new());
}

/// The Lima conf with `[chaotic-aur]` padded with spaces and `[core]`
/// appended once more.
#[test]
fn repos_are_the_sections_besides_options_in_file_order() {
    let conf = parse(&format!(
        "{}[core]\n",
        lima_with(&[("[chaotic-aur]", "  [chaotic-aur]  ")])
    ));

    assert_eq!(repos(&conf), LIMA_REPOS);
}

/// The Lima conf with `RootDir` and `DBPath` set in `[options]` and
/// another `DBPath` put into `[core]`.
#[test]
fn db_path_and_root_dir_come_from_options_only() {
    let conf = parse(&lima_with(&[
        ("#RootDir     = /", "RootDir = /mnt"),
        ("#DBPath      = /var/lib/pacman/", "DBPath=/mnt/db/"),
        ("[core]\n", "[core]\nDBPath = /elsewhere\n"),
    ]));

    assert_eq!(conf.root, PathBuf::from("/mnt"));
    assert_eq!(conf.db_path, PathBuf::from("/mnt/db/"));
}

/// pacman puts the databases below the root when only `RootDir` is set.
/// The Lima conf with `RootDir = /mnt`.
#[test]
fn a_root_dir_moves_the_default_database() {
    let conf = parse(&lima_with(&[("#RootDir     = /", "RootDir     = /mnt")]));

    assert_eq!(conf.db_path, PathBuf::from("/mnt/var/lib/pacman"));
}

/// Hostile section names, made up.
#[test]
fn sections_that_cannot_name_a_database_file_are_skipped() {
    let conf = parse("[local]\n[../etc]\n[a/b]\n[]\n[.hidden]\n[good_one.x]\n");

    assert_eq!(repos(&conf), ["good_one.x"]);
}
