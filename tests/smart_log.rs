mod common;
use common::*;

#[test]
fn echoes_the_git_log_command() {
    let sb = Sandbox::new();
    let repo = sb.path("repo");
    init_repo(&repo);
    let r = run(&repo, "smart-log", &["-n", "1"]);
    r.ok().assert_reports("Executing: git log --pretty=format:");
    r.assert_reports("--graph -n 1");
}
