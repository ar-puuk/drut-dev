//! Unit tests for `drut_config::discover` (012-toml-configuration T007).

use std::path::{Path, PathBuf};

use drut_config::discover;

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("drut_config_discover_test_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn drut_toml_in_the_same_directory_as_the_target_wins() {
    let root = test_dir("same_dir");
    write(&root.join("drut.toml"), "[format]\ncasing_control_words = \"lower\"\n");
    let target = root.join("a.s");
    write(&target, "IF (a=b)\nENDIF\n");

    let found = discover(&target);
    assert_eq!(found, Some(root.join("drut.toml")));

    cleanup(&root);
}

#[test]
fn a_drut_toml_higher_up_wins_when_nothing_closer_exists() {
    let root = test_dir("higher_up");
    write(&root.join("drut.toml"), "[format]\ncasing_control_words = \"lower\"\n");
    let target = root.join("sub").join("a.s");
    write(&target, "IF (a=b)\nENDIF\n");

    let found = discover(&target);
    assert_eq!(found, Some(root.join("drut.toml")));

    cleanup(&root);
}

#[test]
fn target_three_directories_deep_finds_the_project_root_config() {
    // spec.md US1 Acceptance Scenario 4.
    let root = test_dir("three_deep");
    write(&root.join("drut.toml"), "[format]\ncasing_control_words = \"lower\"\n");
    let target = root.join("a").join("b").join("c").join("a.s");
    write(&target, "IF (a=b)\nENDIF\n");

    let found = discover(&target);
    assert_eq!(found, Some(root.join("drut.toml")));

    cleanup(&root);
}

#[test]
fn a_git_directory_between_the_target_and_a_further_drut_toml_stops_the_walk() {
    let root = test_dir("git_dir_boundary");
    write(&root.join("drut.toml"), "[format]\ncasing_control_words = \"lower\"\n");
    let repo = root.join("repo");
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    let target = repo.join("a.s");
    write(&target, "IF (a=b)\nENDIF\n");

    let found = discover(&target);
    assert_eq!(
        found, None,
        "a .git directory must stop the walk before reaching the drut.toml above it"
    );

    cleanup(&root);
}

#[test]
fn a_git_file_worktree_between_the_target_and_a_further_drut_toml_stops_the_walk() {
    // A real git worktree's own `.git` is a *file* (containing a `gitdir:`
    // pointer), not a directory — discover() only needs to detect presence,
    // per its own doc comment. Found missing from the original test list
    // during /speckit-analyze review.
    let root = test_dir("git_file_boundary");
    write(&root.join("drut.toml"), "[format]\ncasing_control_words = \"lower\"\n");
    let repo = root.join("worktree");
    std::fs::create_dir_all(&repo).unwrap();
    write(&repo.join(".git"), "gitdir: /some/where/.git/worktrees/worktree\n");
    let target = repo.join("a.s");
    write(&target, "IF (a=b)\nENDIF\n");

    let found = discover(&target);
    assert_eq!(
        found, None,
        "a .git file (worktree shape) must stop the walk exactly like a .git directory"
    );

    cleanup(&root);
}

#[test]
fn a_git_boundary_with_no_config_inside_it_returns_none() {
    let root = test_dir("no_config_at_all");
    std::fs::create_dir_all(root.join(".git")).unwrap();
    let target = root.join("sub").join("a.s");
    write(&target, "IF (a=b)\nENDIF\n");

    let found = discover(&target);
    assert_eq!(found, None);

    cleanup(&root);
}

#[test]
fn discover_on_a_nonexistent_path_does_not_panic() {
    let path = Path::new("this/definitely/does/not/exist/anywhere/a.s");
    // Never panics -- may or may not find something depending on the real
    // filesystem above the (nonexistent) parent, but must return, not crash.
    let _ = discover(path);
}
