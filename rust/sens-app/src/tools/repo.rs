use serde_json::{Value, json};

use super::{Desk, Level, Said, Scope, Tool, schema, text};
use crate::git;

const MOST_FILES: usize = 200;

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "repo_status",
        title: "Repository status",
        level: Level::Read,
        description: "The git state of the session's working folder as Sens shows it: the branch, the other branches, and the files changed or new since the last commit.",
        input: || schema(json!({}), &[]),
        run: repo_status,
    },
    Tool {
        name: "switch_branch",
        title: "Switch branch",
        level: Level::Change,
        description: "Check out another existing branch in the session's working folder, and refresh Sens's tree, viewer and branch chip with it. Git refuses when uncommitted changes would be lost.",
        input: || schema(json!({ "branch": { "type": "string" } }), &["branch"]),
        run: switch_branch,
    },
];

fn changed(diff: &str) -> Vec<String> {
    diff.lines().filter_map(|line| line.strip_prefix("diff --git ")).filter_map(|paths| paths.rsplit_once(" b/").map(|(_, path)| path.to_string())).collect()
}

fn repo_status(_: &Desk, scope: &Scope, _: &Value) -> Result<Said, String> {
    let work = scope.work();
    let repo = git::read(work).ok_or_else(|| format!("{} is not a git repository", work.display()))?;
    let head = if repo.detached { format!("Detached at {}", repo.branch) } else { format!("On branch {}", repo.branch) };
    let others: Vec<&String> = repo.branches.iter().filter(|branch| **branch != repo.branch).collect();
    let mut lines = vec![head];
    if !others.is_empty() {
        lines.push(format!("Other branches: {}", others.iter().map(|branch| branch.as_str()).collect::<Vec<_>>().join(", ")));
    }
    let changes = git::changes(work);
    let edited = changes.as_ref().map(|changes| changed(&changes.diff)).unwrap_or_default();
    let fresh = changes.map(|changes| changes.fresh).unwrap_or_default();
    if edited.is_empty() && fresh.is_empty() {
        lines.push("Nothing changed since the last commit.".into());
    }
    lines.extend(edited.into_iter().map(|path| format!("  changed  {path}")).chain(fresh.into_iter().map(|path| format!("  new      {path}"))).take(MOST_FILES));
    Ok(lines.join("\n").into())
}

fn switch_branch(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let branch = text(arguments, "branch")?;
    let repo = git::checkout(scope.work(), &branch)?;
    let _ = desk.ui.act("branch_switched", json!({ "session": scope.session, "branch": repo.branch }));
    Ok(format!("Now on {}.", repo.branch).into())
}

#[cfg(test)]
mod tests {
    use super::super::call;
    use super::super::testing::{Window, scope};
    use super::*;
    use crate::terminal::Consoles;
    use sens_agent::canon::keeper::Keeper;
    use std::path::Path;
    use std::process::Command;

    fn git_in(root: &Path, args: &[&str]) {
        let done = Command::new("git").args(["-c", "user.name=Sens", "-c", "user.email=sens@example.com"]).args(args).current_dir(root).output().unwrap();
        assert!(done.status.success(), "{}", String::from_utf8_lossy(&done.stderr));
    }

    #[test]
    fn the_paths_of_a_diff_are_the_files_it_changed() {
        let diff = "diff --git a/src/app.ts b/src/app.ts\nindex 1..2\n--- a/src/app.ts\n+++ b/src/app.ts\ndiff --git a/old.md b/docs/new.md\n";
        assert_eq!(changed(diff), ["src/app.ts", "docs/new.md"]);
    }

    #[test]
    fn a_repository_tells_its_branch_and_changes_and_switches() {
        let root = std::env::temp_dir().join("sens-tools-repo");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        git_in(&root, &["init", "-q", "-b", "main"]);
        std::fs::write(root.join("app.ts"), "uno\n").unwrap();
        git_in(&root, &["add", "-A"]);
        git_in(&root, &["commit", "-q", "-m", "uno"]);
        git_in(&root, &["branch", "rama"]);
        std::fs::write(root.join("app.ts"), "dos\n").unwrap();
        std::fs::write(root.join("nuevo.ts"), "x\n").unwrap();

        let consoles = Consoles::default();
        let keeper = Keeper::default();
        let window = Window::default();
        let desk = window.desk(&consoles, &keeper);
        let here = scope(&root);
        let status = call(&desk, &here, "repo_status", &json!({})).unwrap().unwrap();
        assert_eq!(status, Said::Text("On branch main\nOther branches: rama\n  changed  app.ts\n  new      nuevo.ts".into()));

        git_in(&root, &["stash", "-u", "-q"]);
        assert_eq!(call(&desk, &here, "switch_branch", &json!({ "branch": "rama" })).unwrap(), Ok("Now on rama.".into()));
        assert_eq!(window.asked.lock().unwrap()[0], ("branch_switched".to_string(), json!({ "session": "s1", "branch": "rama" })));
        assert!(call(&desk, &here, "switch_branch", &json!({ "branch": "no-existe" })).unwrap().is_err());
        assert!(call(&desk, &scope(&std::env::temp_dir().join("sens-tools-no-repo-here")), "repo_status", &json!({})).unwrap().unwrap_err().contains("not a git repository"));
    }
}
