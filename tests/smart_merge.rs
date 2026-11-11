mod common;
use common::*;

fn setup() -> (Sandbox, std::path::PathBuf) {
    let sb = Sandbox::new();
    let local = sb.path("local");
    init_repo(&local);
    (sb, local)
}

fn with_newbranch(local: &std::path::Path) {
    git(local, &["checkout", "-q", "-b", "newbranch"]);
    append(local, "README", "moar things!\n");
    write(local, "lib/moar.rb", "puts 'moar'\n");
    commit_all(local, "moar");
    git(local, &["checkout", "-q", "main"]);
}

#[test]
fn requires_an_argument() {
    let (_sb, local) = setup();
    let err = run(&local, "smart-merge", &[])
        .result
        .unwrap_err()
        .to_string();
    assert_eq!(err, "Usage: git smart-merge ref");
}

#[test]
fn requires_a_valid_branch() {
    let (_sb, local) = setup();
    let err = run(&local, "smart-merge", &["foo"])
        .result
        .unwrap_err()
        .to_string();
    assert_eq!(err, "Branch to merge 'foo' not recognised by git!");
}

#[test]
fn reports_nothing_to_do_when_branch_has_not_moved() {
    let (_sb, local) = setup();
    git(&local, &["branch", "unmoved"]);
    let r = run(&local, "smart-merge", &["unmoved"]);
    r.ok()
        .assert_reports("Branch 'unmoved' has no new commits. Nothing to merge in.");
    r.assert_reports("Already up-to-date.");
}

#[test]
fn merges_no_ff_even_when_fast_forwardable() {
    let (_sb, local) = setup();
    with_newbranch(&local);
    let r = run(&local, "smart-merge", &["newbranch"]);
    r.ok()
        .assert_reports("Branch 'newbranch' has diverged by 1 commit. Merging in.");
    r.assert_reports(
        "Branch 'main' has not moved on since 'newbranch' diverged. Running with --no-ff anyway",
    );
    r.assert_reports("Executing: git merge --no-ff newbranch");
    r.assert_reports("All good. Created merge commit");
    let parents = git(&local, &["rev-list", "--parents", "-n", "1", "HEAD"]);
    assert_eq!(
        parents.split_whitespace().count(),
        3,
        "expected a merge commit"
    );
}

#[test]
fn merges_when_both_branches_moved() {
    let (_sb, local) = setup();
    with_newbranch(&local);
    append(&local, "lib/codes.rb", "more\n");
    commit_all(&local, "changes on main");
    let r = run(&local, "smart-merge", &["newbranch"]);
    r.ok()
        .assert_reports("Branch 'newbranch' has diverged by 1 commit. Merging in.");
    r.assert_reports("Branch 'main' has 1 new commit since 'newbranch' diverged.");
    r.assert_reports("Executing: git merge --no-ff newbranch");
}

#[test]
fn stashes_and_restores_a_dirty_tree() {
    let (_sb, local) = setup();
    with_newbranch(&local);
    append(&local, "lib/codes.rb", "more\n");
    commit_all(&local, "changes on main");
    write(&local, "noob", "i am nub\n");
    append(&local, "lib/codes.rb", "dirty\n");
    git(&local, &["add", "noob"]);
    let before = status(&local);
    let r = run(&local, "smart-merge", &["newbranch"]);
    r.ok().assert_reports("Executing: git stash");
    r.assert_reports("Reapplying local changes...");
    r.assert_reports("Executing: git stash pop");
    assert_eq!(status(&local), before);
}

#[test]
fn conflicts_are_reported_and_fail() {
    let (_sb, local) = setup();
    git(&local, &["checkout", "-q", "-b", "newbranch"]);
    write(&local, "README", "newbranch changes\n");
    commit_all(&local, "newbranch_commit");
    git(&local, &["checkout", "-q", "main"]);
    write(&local, "README", "main changes\n");
    commit_all(&local, "main_commit");

    let r = run(&local, "smart-merge", &["newbranch"]);
    r.assert_not_reports("All good");
    r.assert_reports("Executing: git merge --no-ff newbranch");
    let err = r.result.unwrap_err().to_string();
    assert!(
        err.contains("CONFLICT (content): Merge conflict in README"),
        "{err}"
    );
    assert!(err.contains("Automatic merge failed"), "{err}");
    assert_eq!(status(&local), "UU README\n");
}
