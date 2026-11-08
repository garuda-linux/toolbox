use std::fs;
use std::path::{Path, PathBuf};

use super::{parse, read, wildcard_match};

/// The head of Arch's own `pacman.conf`, trailing comments added the way
/// people write them.
const ARCH: &str = "\
#
# /etc/pacman.conf
#
[options]
#RootDir     = /
DBPath      = /var/lib/pacman/   # where the databases live
HoldPkg     = pacman glibc
Architecture = auto
CheckSpace
ParallelDownloads = 5
SigLevel    = Required DatabaseOptional
LocalFileSigLevel = Optional

#[core-testing]
#Include = /etc/pacman.d/mirrorlist

[core] # the base system
Include = /etc/pacman.d/mirrorlist

[extra]
Include = /etc/pacman.d/mirrorlist
";

fn repos(conf: &super::PacmanConf) -> Vec<&str> {
    conf.repos.iter().map(String::as_str).collect()
}

fn write(folder: &Path, name: &str, text: &str) -> PathBuf {
    let path = folder.join(name);

    fs::write(&path, text).unwrap();
    path
}

#[test]
fn trailing_comments_are_cut_like_libalpm_cuts_them() {
    let conf = parse(ARCH);

    assert_eq!(repos(&conf), ["core", "extra"]);
    assert_eq!(conf.db_path, PathBuf::from("/var/lib/pacman/"));
    assert_eq!(conf.root, PathBuf::from("/"));
}

#[test]
fn a_value_with_a_comment_keeps_only_what_comes_before_it() {
    let conf = parse("[options]\nRootDir = /mnt#chroot\n[garuda]#ours\n");

    assert_eq!(conf.root, PathBuf::from("/mnt"));
    assert_eq!(repos(&conf), ["garuda"]);
}

/// Garuda's `pacman.conf` keeps its own repositories in an included file,
/// which pacman accepts.
#[test]
fn repositories_in_an_included_file_count_in_their_place() {
    let folder = tempfile::tempdir().unwrap();
    let mirrors = write(
        folder.path(),
        "mirrorlist",
        "Server = https://mirror/$repo\n",
    );

    let extra = write(
        folder.path(),
        "garuda.conf",
        &format!(
            "[garuda]\nInclude = {}\n\n[chaotic-aur] # builds\nInclude = {}\n",
            mirrors.display(),
            mirrors.display()
        ),
    );

    let main = write(
        folder.path(),
        "pacman.conf",
        &format!(
            "{ARCH}\n[options]\nInclude = {}\n\n[multilib]\nInclude = {}\n",
            extra.display(),
            mirrors.display()
        ),
    );

    let conf = read(&main).unwrap();

    assert_eq!(
        repos(&conf),
        ["core", "extra", "garuda", "chaotic-aur", "multilib"]
    );
}

#[test]
fn wildcard_includes_are_read_in_sorted_order() {
    let folder = tempfile::tempdir().unwrap();
    let drop_in = folder.path().join("repos.d");

    fs::create_dir_all(&drop_in).unwrap();
    write(&drop_in, "20-second.conf", "[second]\n");
    write(&drop_in, "10-first.conf", "[first]\n");
    write(&drop_in, ".hidden.conf", "[hidden]\n");
    write(&drop_in, "notes.txt", "[notes]\n");

    let main = write(
        folder.path(),
        "pacman.conf",
        &format!("[options]\nInclude = {}/*.conf\n", drop_in.display()),
    );

    assert_eq!(repos(&read(&main).unwrap()), ["first", "second"]);
}

#[test]
fn a_missing_include_or_one_that_includes_itself_ends() {
    let folder = tempfile::tempdir().unwrap();
    let main = folder.path().join("pacman.conf");

    fs::write(
        &main,
        format!(
            "[core]\nInclude = /nonexistent/mirrorlist\nInclude = {}\n",
            main.display()
        ),
    )
    .unwrap();

    assert_eq!(repos(&read(&main).unwrap()), ["core"]);
}

#[test]
fn a_stray_byte_hides_no_repository() {
    let folder = tempfile::tempdir().unwrap();
    let main = folder.path().join("pacman.conf");

    fs::write(&main, b"# caf\xe9\n[core]\n[extra]\n").unwrap();

    assert_eq!(repos(&read(&main).unwrap()), ["core", "extra"]);
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

#[test]
fn repos_are_the_sections_besides_options_in_file_order() {
    let conf = parse(
        "# a comment\n[options]\nHoldPkg = pacman glibc\nArchitecture = auto\n\n\
         [core]\nInclude = /etc/pacman.d/mirrorlist\n\n[extra]\n\
         Include = /etc/pacman.d/mirrorlist\n\n  [chaotic-aur]  \n\
         Include = /etc/pacman.d/chaotic-mirrorlist\n[core]\n",
    );

    assert_eq!(repos(&conf), ["core", "extra", "chaotic-aur"]);
}

#[test]
fn db_path_and_root_dir_come_from_options_only() {
    let conf = parse("[options]\nRootDir = /mnt\nDBPath=/mnt/db/\n[core]\nDBPath = /elsewhere\n");

    assert_eq!(conf.root, PathBuf::from("/mnt"));
    assert_eq!(conf.db_path, PathBuf::from("/mnt/db/"));
}

/// pacman puts the databases below the root when only `RootDir` is set.
#[test]
fn a_root_dir_moves_the_default_database() {
    let conf = parse("[options]\nRootDir = /mnt\n");

    assert_eq!(conf.db_path, PathBuf::from("/mnt/var/lib/pacman"));
}

#[test]
fn sections_that_cannot_name_a_database_file_are_skipped() {
    let conf = parse("[local]\n[../etc]\n[a/b]\n[]\n[.hidden]\n[good_one.x]\n");

    assert_eq!(repos(&conf), ["good_one.x"]);
}
