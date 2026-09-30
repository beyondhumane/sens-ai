use std::path::Path;
use std::process::Command;

use sens_canon::dependencies;
use serde_json::Value;

use crate::git;

const LOCKS: &[&str] = &["package-lock.json", "Cargo.lock", "poetry.lock", "yarn.lock", "pnpm-lock.yaml", "uv.lock"];
const IGNORED: &str = "**/node_modules/**,**/target/**,**/.sens/**,**/.git/**,**/accept/**,**/__pycache__/**,**/*.lock,**/package-lock.json";
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

pub fn duplicated_lines(dir: &Path, jscpd: &Path) -> Result<u64, String> {
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
        return Ok(0);
    }
    let parsed: Value = serde_json::from_str(&std::fs::read_to_string(&report).map_err(|error| format!("jscpd: {error}"))?)
        .map_err(|error| format!("jscpd: {error}"))?;
    Ok(parsed["statistics"]["total"]["duplicatedLines"].as_u64().unwrap_or(0))
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
    fn jscpd_finds_a_renamed_copy() {
        let dir = std::env::temp_dir().join("sens-bench-dup");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let body = "function sum(list) {\n  let total = 0;\n  for (const item of list) {\n    if (item > 0) {\n      total += item * 2;\n    } else {\n      total -= item;\n    }\n  }\n  return total;\n}\n";
        std::fs::write(dir.join("one.js"), body).unwrap();
        std::fs::write(dir.join("two.js"), body.replace("sum", "add")).unwrap();
        let jscpd = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../node_modules/jscpd/run-jscpd.js");
        assert!(duplicated_lines(&dir, &jscpd).unwrap() > 0);
        std::fs::remove_file(dir.join("two.js")).unwrap();
        assert_eq!(duplicated_lines(&dir, &jscpd).unwrap(), 0);
    }
}
