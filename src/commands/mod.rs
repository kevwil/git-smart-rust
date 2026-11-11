pub mod log;
pub mod merge;
pub mod pull;

use crate::error::Result;
use crate::git::Repo;
use crate::ui::Ui;
use std::io::Write;

/// Runs `f` with local changes stashed (if any), then restores them.
///
/// If `f` fails, the stash is left in place and the user is told how to get it back.
fn with_stash<W: Write, T>(
    repo: &Repo,
    ui: &mut Ui<W>,
    f: impl FnOnce(&mut Ui<W>) -> Result<T>,
) -> Result<T> {
    if !repo.is_dirty() {
        return f(ui);
    }
    ui.note("Working directory dirty. Stashing...")?;
    repo.run(ui, &["stash"])?;
    let value = match f(ui) {
        Ok(v) => v,
        Err(e) => {
            ui.note("Your local changes are still stashed. Once resolved, run `git stash pop`.")?;
            return Err(e);
        }
    };
    ui.note("Reapplying local changes...")?;
    repo.run(ui, &["stash", "pop"])?;
    Ok(value)
}
