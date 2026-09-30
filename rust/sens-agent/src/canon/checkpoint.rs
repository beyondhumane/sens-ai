use std::path::{Path, PathBuf};
use std::process::Command;

use sens_canon::verdict::Change;
use serde::{Deserialize, Serialize};

use crate::process::hidden;
use crate::session::keep_from_git;

const FOLDER: [&str; 3] = [".sens", "canon", "checkpoints"];
const EXCLUDED: &str = ".sens/\n.git/\n";
const AUTHOR: [&str; 4] = ["-c", "user.name=sens", "-c", "user.email=canon@sens.dev"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Restored {
    pub restored: Vec<String>,
    pub skipped: Vec<String>,
}

pub struct Checkpoints {
    store: PathBuf,
    work: PathBuf,
}

fn plain(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest) => PathBuf::from(rest),
        None => path.to_path_buf(),
    }
}

impl Checkpoints {
    pub fn open(work: &Path) -> Option<Checkpoints> {
        let work = plain(work);
        let store = FOLDER.iter().fold(work.clone(), |path, part| path.join(part));
        let checkpoints = Checkpoints { store, work: work.clone() };
        if !checkpoints.store.join("HEAD").is_file() {
            std::fs::create_dir_all(&checkpoints.store).ok()?;
            keep_from_git(&work);
            checkpoints.git(&["init", "-q"]).ok()?;
            checkpoints.git(&["config", "core.autocrlf", "false"]).ok()?;
            checkpoints.git(&["config", "core.safecrlf", "false"]).ok()?;
            std::fs::create_dir_all(checkpoints.store.join("info")).ok()?;
            std::fs::write(checkpoints.store.join("info").join("exclude"), EXCLUDED).ok()?;
        }
        Some(checkpoints)
    }

    fn command(&self) -> Command {
        let mut command = Command::new("git");
        hidden(&mut command).arg("--git-dir").arg(&self.store).arg("--work-tree").arg(&self.work).args(AUTHOR);
        command
    }

    fn git(&self, args: &[&str]) -> Result<String, String> {
        let done = self.command().args(args).output().map_err(|error| format!("git: {error}"))?;
        if !done.status.success() {
            return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&done.stderr).trim()));
        }
        Ok(String::from_utf8_lossy(&done.stdout).trim().to_string())
    }

    pub fn snapshot(&self, label: &str) -> Result<Tree, String> {
        self.git(&["add", "-A", "--ignore-errors", "."])?;
        let tree = self.git(&["write-tree"])?;
        let commit = self.git(&["commit-tree", &tree, "-m", label])?;
        self.git(&["update-ref", &format!("refs/turns/{label}"), &commit])?;
        Ok(Tree(tree))
    }

    pub fn changed(&self, from: &Tree, to: &Tree) -> Result<Vec<(Status, String)>, String> {
        let listed = self.git(&["diff-tree", "-r", "--no-renames", "--name-status", &from.0, &to.0])?;
        Ok(listed
            .lines()
            .filter_map(|line| {
                let (status, path) = line.split_once('\t')?;
                let status = match status {
                    "A" => Status::Added,
                    "D" => Status::Deleted,
                    _ => Status::Modified,
                };
                Some((status, path.to_string()))
            })
            .collect())
    }

    pub fn read(&self, tree: &Tree, path: &str) -> Option<Vec<u8>> {
        let done = self.command().args(["cat-file", "blob", &format!("{}:{path}", tree.0)]).output().ok()?;
        done.status.success().then_some(done.stdout)
    }

    pub fn diff(&self, from: &Tree, to: &Tree) -> Result<String, String> {
        self.git(&["diff", "--no-color", "--no-ext-diff", "--no-renames", "-U3", &from.0, &to.0])
    }

    pub fn changes(&self, from: &Tree, to: &Tree) -> Result<Vec<Change>, String> {
        let text = |tree: &Tree, path: &str| self.read(tree, path).and_then(|bytes| String::from_utf8(bytes).ok());
        Ok(self
            .changed(from, to)?
            .into_iter()
            .filter_map(|(status, path)| {
                let before = (status != Status::Added).then(|| text(from, &path)).flatten();
                let after = (status != Status::Deleted).then(|| text(to, &path)).flatten();
                (before.is_some() || after.is_some()).then_some(Change { path, before, after })
            })
            .collect())
    }

    pub fn put_back(&self, tree: &Tree, path: &str) -> Result<(), String> {
        let target = self.work.join(path);
        match self.read(tree, path) {
            Some(bytes) => {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
                }
                std::fs::write(&target, bytes).map_err(|error| format!("{}: {error}", target.display()))
            }
            None if target.exists() => std::fs::remove_file(&target).map_err(|error| format!("{}: {error}", target.display())),
            None => Ok(()),
        }
    }

    pub fn restore(&self, from: &Tree, end: &Tree) -> Result<Restored, String> {
        let mut restored = Restored::default();
        for (_, path) in self.changed(from, end)? {
            let now = std::fs::read(self.work.join(&path)).ok();
            if now != self.read(end, &path) {
                restored.skipped.push(path);
                continue;
            }
            self.put_back(from, &path)?;
            restored.restored.push(path);
        }
        Ok(restored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join("sens-checkpoints").join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn put(root: &Path, path: &str, bytes: &[u8]) {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, bytes).unwrap();
    }

    #[test]
    fn a_verbatim_windows_path_is_read_as_a_plain_one() {
        assert_eq!(plain(Path::new(r"\\?\P:\Sens")), PathBuf::from(r"P:\Sens"));
        assert_eq!(plain(Path::new("/tmp/x")), PathBuf::from("/tmp/x"));
    }

    #[test]
    fn two_snapshots_tell_what_changed_and_what_was_there() {
        let root = folder("changed");
        put(&root, "a.txt", b"one\n");
        put(&root, "c.txt", b"gone\n");
        put(&root, ".sens/sessions/x.jsonl", b"private\n");
        let checkpoints = Checkpoints::open(&root).unwrap();
        let start = checkpoints.snapshot("s/1").unwrap();
        put(&root, "a.txt", b"two\n");
        put(&root, "src/b.ts", b"export const b = 1;\n");
        std::fs::remove_file(root.join("c.txt")).unwrap();
        let end = checkpoints.snapshot("s/2").unwrap();
        let mut changed = checkpoints.changed(&start, &end).unwrap();
        changed.sort_by(|a, b| a.1.cmp(&b.1));
        assert_eq!(changed, [(Status::Modified, "a.txt".to_string()), (Status::Deleted, "c.txt".to_string()), (Status::Added, "src/b.ts".to_string())]);
        assert_eq!(checkpoints.read(&start, "a.txt").as_deref(), Some(&b"one\n"[..]));
        assert_eq!(checkpoints.read(&start, ".sens/sessions/x.jsonl"), None);
        let changes = checkpoints.changes(&start, &end).unwrap();
        assert!(changes.iter().any(|change| change.path == "src/b.ts" && change.before.is_none() && change.after.as_deref() == Some("export const b = 1;\n")));
    }

    #[test]
    fn undoing_puts_every_byte_back_and_leaves_the_person_s_later_edit_alone() {
        let root = folder("restore");
        put(&root, "crlf.txt", b"one\r\ntwo\r\n");
        put(&root, "binary.bin", &[0, 159, 255, 10]);
        put(&root, "empty.txt", b"");
        put(&root, "mine.txt", b"base\n");
        let checkpoints = Checkpoints::open(&root).unwrap();
        let start = checkpoints.snapshot("s/1").unwrap();
        put(&root, "crlf.txt", b"one\ntwo\n");
        put(&root, "binary.bin", &[1]);
        std::fs::remove_file(root.join("empty.txt")).unwrap();
        put(&root, "new/made.txt", b"made\n");
        put(&root, "mine.txt", b"turn\n");
        let end = checkpoints.snapshot("s/2").unwrap();
        put(&root, "mine.txt", b"person\n");

        let mut restored = checkpoints.restore(&start, &end).unwrap();
        restored.restored.sort();
        assert_eq!(restored.restored, ["binary.bin", "crlf.txt", "empty.txt", "new/made.txt"]);
        assert_eq!(restored.skipped, ["mine.txt"]);
        assert_eq!(std::fs::read(root.join("crlf.txt")).unwrap(), b"one\r\ntwo\r\n");
        assert_eq!(std::fs::read(root.join("binary.bin")).unwrap(), [0, 159, 255, 10]);
        assert_eq!(std::fs::read(root.join("empty.txt")).unwrap(), b"");
        assert!(!root.join("new/made.txt").exists());
        assert_eq!(std::fs::read(root.join("mine.txt")).unwrap(), b"person\n");
    }

    #[test]
    fn the_project_s_own_repository_is_never_touched() {
        let root = folder("own-git");
        put(&root, ".gitignore", b"build/\n");
        put(&root, "a.txt", b"one\n");
        put(&root, "build/out.txt", b"ignored\n");
        let git = |args: &[&str]| {
            let done = Command::new("git").arg("-C").arg(&root).args(AUTHOR).args(args).output().unwrap();
            assert!(done.status.success(), "{}", String::from_utf8_lossy(&done.stderr));
            String::from_utf8_lossy(&done.stdout).to_string()
        };
        git(&["init", "-q"]);
        git(&["add", "-A"]);
        git(&["commit", "-qm", "base"]);
        put(&root, "a.txt", b"staged\n");
        git(&["add", "a.txt"]);
        let state = || (git(&["ls-files", "-s"]), git(&["rev-parse", "HEAD"]), git(&["status", "--porcelain", "--untracked-files=all"]), git(&["stash", "list"]), git(&["branch", "--list"]));
        let before = state();

        let checkpoints = Checkpoints::open(&root).unwrap();
        let start = checkpoints.snapshot("s/1").unwrap();
        put(&root, "a.txt", b"turn\n");
        let end = checkpoints.snapshot("s/2").unwrap();
        assert_eq!(checkpoints.read(&start, "build/out.txt"), None);
        checkpoints.restore(&start, &end).unwrap();

        assert_eq!(before, state());
        assert_eq!(std::fs::read(root.join("a.txt")).unwrap(), b"staged\n");
    }
}
