use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use sens_canon::dependencies;
use sens_index::build;
use sens_index::testfile::is_test_file;
use serde_json::Value;

use crate::git;

const LOCKS: &[&str] = &["package-lock.json", "Cargo.lock", "poetry.lock", "yarn.lock", "pnpm-lock.yaml", "uv.lock"];
const IGNORED: &str = "**/node_modules/**,**/target/**,**/.sens/**,**/.git/**,**/accept/**,**/_accept/**,**/__pycache__/**,**/*.lock,**/package-lock.json";
const MIN_TOKENS: &str = "30";

fn locked(path: &str) -> bool {
    LOCKS.iter().any(|lock| path == *lock || path.ends_with(&format!("/{lock}")))
}

#[derive(Debug, Default, PartialEq)]
pub struct Lines {
    pub added: u64,
    pub removed: u64,
}

pub fn lines(numstat: &str) -> Lines {
    numstat
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let added = fields.next()?.parse::<u64>().ok()?;
            let removed = fields.next()?.parse::<u64>().ok()?;
            (!locked(fields.next()?)).then_some(Lines { added, removed })
        })
        .fold(Lines::default(), |total, file| Lines {
            added: total.added + file.added,
            removed: total.removed + file.removed,
        })
}

#[derive(Default)]
struct Side {
    path: String,
    whole_file: bool,
    ranges: Vec<(u32, u32)>,
    line: u32,
}

impl Side {
    fn of(path: &str, content: String) -> Side {
        let whole_file = is_test_file(path);
        let ranges = match whole_file {
            true => Vec::new(),
            false => build::analyze(path, &content).into_iter().filter(|unit| unit.whole && unit.test).map(|unit| (unit.start_line, unit.end_line)).collect(),
        };
        Side { path: path.to_string(), whole_file, ranges, line: 0 }
    }

    fn holds_test(&self) -> bool {
        !self.path.is_empty() && !locked(&self.path) && (self.whole_file || self.ranges.iter().any(|&(start, end)| (start..=end).contains(&self.line)))
    }
}

fn hunk_start(part: &str) -> u32 {
    part[1..].split(',').next().and_then(|number| number.parse().ok()).unwrap_or(0)
}

pub fn test_lines(patch: &str, before: impl Fn(&str) -> String, after: impl Fn(&str) -> String) -> Lines {
    let (mut old, mut new) = (Side::default(), Side::default());
    let mut total = Lines::default();
    for line in patch.lines() {
        if line.starts_with("diff --git") {
            (old, new) = (Side::default(), Side::default());
        } else if let Some(path) = line.strip_prefix("--- a/") {
            old = Side::of(path, before(path));
        } else if let Some(path) = line.strip_prefix("+++ b/") {
            new = Side::of(path, after(path));
        } else if line.starts_with("--- ") || line.starts_with("+++ ") {
            continue;
        } else if let Some(header) = line.strip_prefix("@@ ") {
            let mut parts = header.split_whitespace();
            old.line = parts.next().map_or(0, hunk_start);
            new.line = parts.next().map_or(0, hunk_start);
        } else if line.starts_with('+') {
            total.added += u64::from(new.holds_test());
            new.line += 1;
        } else if line.starts_with('-') {
            total.removed += u64::from(old.holds_test());
            old.line += 1;
        } else if line.starts_with(' ') {
            old.line += 1;
            new.line += 1;
        }
    }
    total
}

pub fn statuses(name_status: &str) -> Vec<(char, String)> {
    name_status
        .lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            let status = fields.next()?.chars().next()?;
            let path = fields.next_back()?.to_string();
            Some((status, path))
        })
        .collect()
}

pub fn new_files(statuses: &[(char, String)]) -> Vec<String> {
    statuses.iter().filter(|(status, path)| *status == 'A' && !locked(path)).map(|(_, path)| path.clone()).collect()
}

pub fn dependencies_added(dir: &Path, statuses: &[(char, String)]) -> Vec<String> {
    statuses
        .iter()
        .filter(|(status, path)| *status != 'D' && dependencies::is_manifest(path))
        .flat_map(|(_, path)| {
            let after = std::fs::read_to_string(dir.join(path)).unwrap_or_default();
            dependencies::added(path, &git::at_base(dir, path), &after)
        })
        .collect()
}

pub fn reused(patch: &str, symbols: &[String]) -> Vec<String> {
    let added: Vec<&str> = patch.lines().filter(|line| line.starts_with('+') && !line.starts_with("+++")).collect();
    symbols.iter().filter(|symbol| added.iter().any(|line| mentions(line, symbol))).cloned().collect()
}

fn mentions(line: &str, symbol: &str) -> bool {
    let part = |c: char| c.is_alphanumeric() || c == '_';
    line.match_indices(symbol).any(|(at, _)| {
        let before = line[..at].chars().next_back();
        let after = line[at + symbol.len()..].chars().next();
        !before.is_some_and(part) && !after.is_some_and(part)
    })
}

fn clones(dir: &Path, jscpd: &Path) -> Result<Value, String> {
    let out = std::env::temp_dir().join(format!("sens-bench-jscpd-{}", std::process::id())).join(dir.file_name().unwrap_or_default());
    let _ = std::fs::remove_dir_all(&out);
    let done = Command::new("node")
        .arg(jscpd)
        .arg(dir)
        .args(["--silent", "--reporters", "json", "--min-tokens", MIN_TOKENS, "--ignore", IGNORED, "--output"])
        .arg(&out)
        .output()
        .map_err(|error| format!("jscpd: {error}"))?;
    if !done.status.success() {
        return Err(format!("jscpd: {}", String::from_utf8_lossy(&done.stderr).trim()));
    }
    let report = out.join("jscpd-report.json");
    if !report.is_file() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&std::fs::read_to_string(&report).map_err(|error| format!("jscpd: {error}"))?).map_err(|error| format!("jscpd: {error}"))
}

pub fn duplicated_lines(dir: &Path, jscpd: &Path) -> Result<u64, String> {
    Ok(clones(dir, jscpd)?["statistics"]["total"]["duplicatedLines"].as_u64().unwrap_or(0))
}

fn inside(dir: &Path, name: &str) -> String {
    let flat = |text: &str| text.replace('\\', "/");
    let (name, dir) = (flat(name), flat(&dir.to_string_lossy()));
    let folder = dir.rsplit('/').next().unwrap_or_default();
    match name.find(&format!("{folder}/")) {
        Some(at) if name.starts_with(&dir) || name[..at].ends_with('/') || at == 0 => name[at + folder.len() + 1..].to_string(),
        _ => name.trim_start_matches("./").to_string(),
    }
}

pub fn duplicated_added(dir: &Path, jscpd: &Path, patch: &str) -> Result<u64, String> {
    let added: HashSet<(String, u32)> = sens_canon::review::sides(patch).into_iter().flat_map(|(file, sides)| sides.added.into_iter().map(move |(line, _)| (file.clone(), line))).collect();
    if added.is_empty() {
        return Ok(0);
    }
    let found = clones(dir, jscpd)?;
    let mut copied: HashSet<(String, u32)> = HashSet::new();
    for clone in found["duplicates"].as_array().into_iter().flatten() {
        for side in [&clone["firstFile"], &clone["secondFile"]] {
            let file = inside(dir, side["name"].as_str().unwrap_or_default());
            let (start, end) = (side["start"].as_u64().unwrap_or(0) as u32, side["end"].as_u64().unwrap_or(0) as u32);
            copied.extend((start..=end).map(|line| (file.clone(), line)).filter(|place| added.contains(place)));
        }
    }
    Ok(copied.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_add_up_without_lockfiles_or_binaries() {
        let numstat = "3\t1\tsrc/a.ts\n120\t0\tpackage-lock.json\n-\t-\tlogo.png\n2\t2\tweb/Cargo.lock\n4\t0\tsrc/b.ts\n";
        assert_eq!(lines(numstat), Lines { added: 7, removed: 1 });
    }

    #[test]
    fn lines_in_test_files_count_as_tests() {
        let patch = "diff --git a/src/a.py b/src/a.py
--- a/src/a.py
+++ b/src/a.py
@@ -1,1 +1,1 @@
+x = 1
-y = 2
diff --git a/tests/test_a.py b/tests/test_a.py
--- a/tests/test_a.py
+++ b/tests/test_a.py
@@ -1,1 +1,2 @@
+def test_x():
+    assert x == 1
-old = 0
";
        assert_eq!(test_lines(patch, |_| String::new(), |_| String::new()), Lines { added: 2, removed: 1 });
    }

    #[test]
    fn lines_inside_a_rust_test_module_count_as_tests_and_the_rest_as_code() {
        let after = "pub fn quiet(flag: bool) -> bool {
    flag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_is_kept() {
        assert!(quiet(true));
        assert!(!quiet(false));
    }
}
";
        let patch = "diff --git a/src/config.rs b/src/config.rs
--- a/src/config.rs
+++ b/src/config.rs
@@ -1,6 +1,14 @@
 pub fn quiet(flag: bool) -> bool {
+    flag
 }
 
 #[cfg(test)]
 mod tests {
     use super::*;
 
+    #[test]
+    fn quiet_is_kept() {
+        assert!(quiet(true));
+        assert!(!quiet(false));
+    }
 }
";
        assert_eq!(test_lines(patch, |_| String::new(), |_| after.to_string()), Lines { added: 4, removed: 0 });
    }

    #[test]
    fn new_files_are_the_added_ones_that_are_not_locks() {
        let found = statuses("A\tsrc/new.ts\nM\tsrc/old.ts\nA\tCargo.lock\nR100\tsrc/x.ts\tsrc/y.ts\n");
        assert_eq!(found[3], ('R', "src/y.ts".to_string()));
        assert_eq!(new_files(&found), ["src/new.ts"]);
    }

    #[test]
    fn a_symbol_counts_as_reused_only_as_a_whole_word_on_an_added_line() {
        let patch = "+++ b/src/list.ts\n+import { formatBytes } from \"./lib/format.ts\";\n-const old = formatBytesOld(1);\n+const size = myformatBytes(2);\n";
        assert_eq!(reused(patch, &["formatBytes".into(), "dayjs".into()]), ["formatBytes"]);
        assert!(reused("+const size = myformatBytes(2);\n", &["formatBytes".into()]).is_empty());
    }

    #[test]
    fn only_added_lines_that_repeat_code_count_as_added_duplication() {
        let dir = std::env::temp_dir().join("sens-bench-dup");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let body = "function sum(list) {\n  let total = 0;\n  for (const item of list) {\n    if (item > 0) {\n      total += item * 2;\n    } else {\n      total -= item;\n    }\n  }\n  return total;\n}\n";
        std::fs::write(dir.join("one.js"), body).unwrap();
        std::fs::write(dir.join("two.js"), body.replace("sum", "add")).unwrap();
        let jscpd = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../node_modules/jscpd/run-jscpd.js");
        let written: String = body.replace("sum", "add").lines().map(|line| format!("+{line}\n")).collect();
        let copy = format!("+++ b/two.js\n@@ -0,0 +1,11 @@\n{written}");
        assert!(duplicated_added(&dir, &jscpd, &copy).unwrap() >= 8);
        let elsewhere = "+++ b/three.js\n@@ -0,0 +1 @@\n+export const three = 3;\n";
        std::fs::write(dir.join("three.js"), "export const three = 3;\n").unwrap();
        assert_eq!(duplicated_added(&dir, &jscpd, elsewhere).unwrap(), 0);
    }

    #[test]
    fn a_clone_is_matched_to_the_run_folder_whatever_path_jscpd_prints() {
        let dir = Path::new("C:/Temp/sens-bench/hard/task-C0-1");
        assert_eq!(inside(dir, "C:\\Temp\\sens-bench\\hard\\task-C0-1\\src\\a.ts"), "src/a.ts");
        assert_eq!(inside(dir, "task-C0-1\\src\\a.ts"), "src/a.ts");
        assert_eq!(inside(dir, "src\\a.ts"), "src/a.ts");
    }
}
