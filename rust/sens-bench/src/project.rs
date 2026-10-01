use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use regex::Regex;
use sens_index::index::Index;
use sens_index::testfile::is_test_file;
use serde::{Deserialize, Serialize};

use crate::measure;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub lines: u64,
    pub files: u64,
    pub functions: u64,
    pub duplicated: u64,
    pub copies: u64,
    pub dead: u64,
    pub probes: BTreeMap<String, u64>,
}

pub struct Probe {
    pub name: String,
    pub pattern: Regex,
}

fn counted(index: &Index, source: &str) -> Vec<usize> {
    let folder = format!("{source}/");
    index
        .units
        .iter()
        .enumerate()
        .filter(|(_, unit)| unit.whole && !unit.test && unit.file.starts_with(&folder) && !is_test_file(&unit.file))
        .map(|(at, _)| at)
        .collect()
}

fn copied(index: &Index, mine: &[usize]) -> u64 {
    let ours: BTreeSet<usize> = mine.iter().copied().collect();
    mine.iter()
        .filter(|&&at| {
            let unit = &index.units[at];
            let near = if unit.comparable() { index.similar(&unit.print) } else if unit.small() { index.small_like(&unit.print) } else { Vec::new() };
            near.iter().any(|found| found.unit != at && ours.contains(&found.unit))
        })
        .count() as u64
}

fn places(index: &Index, files: &[(String, String)], mine: &[usize], pattern: &Regex) -> u64 {
    let mut found: BTreeSet<(String, u32)> = BTreeSet::new();
    for (path, text) in files {
        for (at, _) in text.lines().enumerate().filter(|(_, line)| pattern.is_match(line)) {
            let number = at as u32 + 1;
            let owner = mine
                .iter()
                .map(|&unit| &index.units[unit])
                .filter(|unit| &unit.file == path && unit.start_line <= number && number <= unit.end_line)
                .min_by_key(|unit| unit.end_line - unit.start_line);
            found.insert((path.clone(), owner.map_or(number, |unit| unit.start_line)));
        }
    }
    found.len() as u64
}

pub fn shape(work: &Path, source: &str, jscpd: &Path, probes: &[Probe]) -> Result<Shape, String> {
    let index = sens_index::build::build(work);
    let folder = format!("{source}/");
    let files: Vec<(String, String)> = index
        .files
        .iter()
        .filter(|file| file.path.starts_with(&folder) && !is_test_file(&file.path))
        .map(|file| (file.path.clone(), std::fs::read_to_string(work.join(&file.path)).unwrap_or_default()))
        .collect();
    let mine = counted(&index, source);
    let dead = sens_canon::orphans::dead(&index).into_iter().filter(|(file, _)| file.starts_with(&folder) && !is_test_file(file)).count() as u64;
    Ok(Shape {
        lines: files.iter().map(|(_, text)| text.lines().filter(|line| !line.trim().is_empty()).count() as u64).sum(),
        files: files.len() as u64,
        functions: mine.iter().filter(|&&at| index.units[at].callable).count() as u64,
        duplicated: measure::duplicated_lines(&work.join(source), jscpd)?,
        copies: copied(&index, &mine),
        dead,
        probes: probes.iter().map(|probe| (probe.name.clone(), places(&index, &files, &mine, &probe.pattern))).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
        let root = std::env::temp_dir().join("sens-bench-project").join(name);
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        root
    }

    const MONEY: &str = "const euros = new Intl.NumberFormat(\"es-ES\", { style: \"currency\", currency: \"EUR\" });\n\nexport function formatMoney(cents: number): string {\n  return euros.format(cents / 100);\n}\n";

    #[test]
    fn a_probe_counts_the_places_that_do_the_same_thing_once_each() {
        let again = "export function price(cents: number): string {\n  const shown = new Intl.NumberFormat(\"es-ES\", { style: \"currency\", currency: \"EUR\" });\n  return shown.format(cents / 100) + new Intl.NumberFormat(\"es-ES\").format(0);\n}\n";
        let root = project("probes", &[("package.json", r#"{ "main": "src/main.ts" }"#), ("src/money.ts", MONEY), ("src/report.ts", again), ("src/money.test.ts", "new Intl.NumberFormat();\n")]);
        let index = sens_index::build::build(&root);
        let mine = counted(&index, "src");
        let files: Vec<(String, String)> = ["src/money.ts", "src/report.ts"].iter().map(|path| (path.to_string(), std::fs::read_to_string(root.join(path)).unwrap())).collect();
        let pattern = Regex::new(r"Intl\.NumberFormat").unwrap();
        assert_eq!(places(&index, &files, &mine, &pattern), 2);
    }

    #[test]
    fn a_function_written_twice_counts_as_two_copies_and_one_alone_as_none() {
        let total = "export function total(rows: { amount: number; active: boolean }[], limit: number) {\n  let sum = 0;\n  for (const row of rows) {\n    if (!row.active || row.amount > limit) {\n      continue;\n    }\n    sum += row.amount;\n  }\n  return sum;\n}\n";
        let root = project("copies", &[("src/a.ts", total), ("src/b.ts", &total.replace("total", "summed")), ("src/c.ts", MONEY)]);
        let index = sens_index::build::build(&root);
        assert_eq!(copied(&index, &counted(&index, "src")), 2);
    }
}
