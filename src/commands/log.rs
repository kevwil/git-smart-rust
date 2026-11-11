use crate::error::Result;
use crate::git::Repo;
use crate::ui::Ui;
use std::io::Write;

const FORMAT: &str = "--pretty=format:%C(yellow)%h%Cblue%d%Creset %s %C(white) %an, %ar%Creset";

/// A very readable `git log --graph`; extra arguments pass straight through.
pub fn run<W: Write>(repo: &Repo, args: &[String], ui: &mut Ui<W>) -> Result<()> {
    let mut git_args = vec!["log", FORMAT, "--graph"];
    git_args.extend(args.iter().map(String::as_str));
    repo.run_inherited(ui, &git_args)
}
