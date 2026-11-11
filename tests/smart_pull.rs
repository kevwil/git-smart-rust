mod common;
use common::*;

/// remote/ and a clone of it in local/.
fn setup() -> (Sandbox, std::path::PathBuf, std::path::PathBuf) {
    let sb = Sandbox::new();
    let (remote, local) = (sb.path("remote"), sb.path("local"));
    init_repo(&remote);
    clone_repo(&remote, &local);
    (sb, remote, local)
}

#[test]
fn nothing_to_do() {
    let (_sb, _remote, local) = setup();
    let r = run(&local, "smart-pull", &[]);
    r.ok().assert_reports("Executing: git fetch origin");
    r.assert_reports(
        "Neither your local branch 'main', nor the remote branch 'origin/main' have moved on.",
    );
    r.assert_reports("Already up-to-date");
}

#[test]
fn only_local_changes() {
    let (_sb, _remote, local) = setup();
    append(&local, "README", "moar\n");
    commit_all(&local, "moar");
    let r = run(&local, "smart-pull", &[]);
    r.ok()
        .assert_reports("Remote branch 'origin/main' has not moved on.");
    r.assert_reports("You have 1 new commit on 'main'.");
    r.assert_reports("Already up-to-date");
}

#[test]
fn ignores_arguments_with_a_warning() {
    let (_sb, _remote, local) = setup();
    let r = run(&local, "smart-pull", &["extra"]);
    r.ok().assert_reports("Ignoring arguments: [\"extra\"]");
}

fn remote_moves_on(remote: &std::path::Path) {
    append(remote, "README", "changed on the server!\n");
    commit_all(remote, "upstream changes");
}

#[test]
fn fast_forwards_when_only_remote_changed() {
    let (_sb, remote, local) = setup();
    remote_moves_on(&remote);
    let r = run(&local, "smart-pull", &[]);
    r.ok()
        .assert_reports("There is 1 new commit on 'origin/main'.");
    r.assert_reports("Local branch 'main' has not moved on. Fast-forwarding...");
    r.assert_reports("Executing: git merge --ff-only origin/main");
    r.assert_reports("Fast forwarded from");
    r.assert_reports("All good.");
    assert_eq!(last_commits(&local, 2), ["upstream changes", "first"]);
}

#[test]
fn does_not_stash_for_untracked_files() {
    let (_sb, remote, local) = setup();
    remote_moves_on(&remote);
    write(&local, "noob", "i am nub\n");
    let r = run(&local, "smart-pull", &[]);
    r.ok().assert_not_reports("Stashing");
    assert_eq!(status(&local), "?? noob\n");
}

#[test]
fn stashes_and_restores_local_changes_around_a_fast_forward() {
    let (_sb, remote, local) = setup();
    remote_moves_on(&remote);
    write(&local, "noob", "i am nub\n");
    append(&local, "lib/codes.rb", "more\n");
    git(&local, &["add", "noob"]);
    let before = status(&local);
    let r = run(&local, "smart-pull", &[]);
    r.ok()
        .assert_reports("Working directory dirty. Stashing...");
    r.assert_reports("Executing: git stash");
    r.assert_reports("Reapplying local changes...");
    r.assert_reports("Executing: git stash pop");
    assert_eq!(status(&local), before);
}

fn diverge(remote: &std::path::Path, local: &std::path::Path) {
    remote_moves_on(remote);
    append(local, "lib/codes.rb", "local\n");
    commit_all(local, "local changes");
}

#[test]
fn rebases_when_both_sides_moved() {
    let (_sb, remote, local) = setup();
    diverge(&remote, &local);
    let r = run(&local, "smart-pull", &[]);
    r.ok()
        .assert_reports("There is 1 new commit on 'origin/main'.");
    r.assert_reports("You have 1 new commit on 'main'.");
    r.assert_reports("Both local and remote branches have moved on. Branch 'main' needs to be rebased onto 'origin/main'");
    r.assert_reports("Executing: git rebase --rebase-merges origin/main");
    r.assert_reports("HEAD moved from");
    assert_eq!(
        last_commits(&local, 3),
        ["local changes", "upstream changes", "first"]
    );
}

#[test]
fn stashes_around_a_rebase() {
    let (_sb, remote, local) = setup();
    diverge(&remote, &local);
    write(&local, "staged", "uncommitted\n");
    git(&local, &["add", "staged"]);
    let before = status(&local);
    let r = run(&local, "smart-pull", &[]);
    r.ok().assert_reports("Executing: git stash pop");
    assert_eq!(status(&local), before);
    assert_eq!(
        last_commits(&local, 3),
        ["local changes", "upstream changes", "first"]
    );
}

#[test]
fn reminds_about_stash_when_rebase_conflicts() {
    let (_sb, remote, local) = setup();
    append(&remote, "README", "server\n");
    commit_all(&remote, "server edit");
    append(&local, "README", "local\n");
    commit_all(&local, "local edit");
    append(&local, "lib/codes.rb", "dirty\n");
    let r = run(&local, "smart-pull", &[]);
    assert!(r.result.is_err());
    r.assert_reports("Your local changes are still stashed");
}

#[test]
fn fails_when_upstream_is_missing() {
    let (_sb, _remote, local) = setup();
    git(&local, &["checkout", "-q", "-b", "topic"]);
    git(&local, &["config", "branch.topic.remote", "origin"]);
    let r = run(&local, "smart-pull", &[]);
    r.assert_reports("No tracking branch configured, assuming 'topic'");
    let err = r.result.unwrap_err().to_string();
    assert!(
        err.contains("Upstream branch 'origin/topic' doesn't exist!"),
        "{err}"
    );
}

#[test]
fn works_inside_a_submodule() {
    let (sb, _remote, local) = setup();
    let sub = sb.path("submodule");
    init_repo(&sub);
    git(
        &local,
        &["submodule", "add", "-q", sub.to_str().unwrap(), "submodule"],
    );
    git(&local, &["commit", "-q", "-m", "Add submodule"]);

    let r = run(&local, "smart-pull", &[]);
    r.ok().assert_reports("You have 1 new commit on 'main'.");

    // The submodule has no remote tracking config of its own beyond origin.
    let r = run(&local.join("submodule"), "smart-pull", &[]);
    r.ok().assert_reports("Already up-to-date");
}

#[test]
fn works_from_a_subdirectory() {
    let (_sb, _remote, local) = setup();
    run(&local.join("lib"), "smart-pull", &[])
        .ok()
        .assert_reports("Already up-to-date");
}
