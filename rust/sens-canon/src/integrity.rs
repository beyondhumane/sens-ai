use crate::verdict::{Change, Finding, Rule, Severity};

const PROTECTED_FOLDERS: [&str; 2] = [".sens", ".git"];
const WORKTREE: &str = "git worktree";

fn settings(name: &str) -> bool {
    name == ".mcp.json" || (name.starts_with("settings") && name.ends_with(".json"))
}

pub fn protected(path: &str) -> bool {
    let parts: Vec<&str> = path.split(['/', '\\']).filter(|part| !part.is_empty() && *part != ".").collect();
    let Some(name) = parts.last() else {
        return false;
    };
    parts.iter().any(|part| PROTECTED_FOLDERS.contains(part)) || *name == ".mcp.json" || (parts.len() >= 2 && parts[parts.len() - 2] == ".claude" && settings(name))
}

fn mentioned(command: &str) -> Option<String> {
    let flat = command.replace('\\', "/");
    let bounded = |at: usize, length: usize| {
        let before = flat[..at].chars().next_back();
        let after = flat[at + length..].chars().next();
        !before.is_some_and(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-') && !after.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.')
    };
    for folder in PROTECTED_FOLDERS {
        if flat.match_indices(folder).any(|(at, _)| bounded(at, folder.len())) {
            return Some(folder.to_string());
        }
    }
    for name in [".mcp.json", ".claude/settings"] {
        if flat.contains(name) {
            return Some(name.to_string());
        }
    }
    None
}

fn finding(file: &str, message: String, key: String) -> Finding {
    Finding { rule: Rule::R7, severity: Severity::Block, file: file.to_string(), line: 1, message, target: None, key }
}

pub fn change(change: &Change) -> Vec<Finding> {
    if !protected(&change.path) {
        return Vec::new();
    }
    vec![finding(
        &change.path,
        format!("{} belongs to Sens, to git or to the agent's configuration, and the agent cannot change it. Leave it as it is and do the task another way.", change.path),
        format!("R7:{}", change.path),
    )]
}

pub fn command(command: &str) -> Vec<Finding> {
    if command.contains(WORKTREE) {
        return vec![finding("", "Sens manages worktrees itself. Work in this folder instead of creating another one.".into(), "R7:worktree".into())];
    }
    match mentioned(command) {
        Some(thing) => vec![finding("", format!("This command touches {thing}, which the agent cannot change. Do the task without it."), format!("R7:{thing}"))],
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sens_git_and_agent_settings_are_protected_and_nothing_else() {
        for path in [".sens/canon/exceptions.json", ".git/hooks/pre-commit", "sub/.git/config", ".mcp.json", ".claude/settings.json", ".claude/settings.local.json", ".\\.claude\\settings.json"] {
            assert!(protected(path), "{path}");
        }
        for path in [".github/workflows/ci.yml", ".gitignore", "src/settings.json", ".claude/agents/reviewer.md", "docs/.sensible.md", "CLAUDE.md"] {
            assert!(!protected(path), "{path}");
        }
    }

    #[test]
    fn a_command_is_blocked_only_when_it_names_a_protected_place() {
        for line in ["echo x > .claude/settings.local.json", "rm -rf .sens", "cp hook .git/hooks/pre-commit", "type .mcp.json", "del .sens\\canon\\exceptions.json", "git worktree add ../elsewhere"] {
            assert!(!command(line).is_empty(), "{line}");
        }
        for line in ["git status", "git add -A && git commit -m x", "cat .gitignore", "ls .github", "npm test", "node scripts/.senseless.js"] {
            assert!(command(line).is_empty(), "{line}");
        }
    }

    #[test]
    fn a_protected_change_is_blocked_whatever_it_writes() {
        let found = change(&Change { path: ".claude/settings.local.json".into(), before: None, after: Some("{}".into()) });
        assert_eq!((found[0].rule, found[0].severity), (Rule::R7, Severity::Block));
        assert!(change(&Change { path: "src/a.ts".into(), before: None, after: Some("x".into()) }).is_empty());
    }
}
