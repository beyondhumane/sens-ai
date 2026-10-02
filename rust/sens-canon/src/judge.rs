use std::collections::{HashMap, HashSet};

use sens_index::build;
use sens_index::index::Index;

use crate::orphans::{self, Dead};
use crate::verdict::{Change, Exceptions, Finding, ProjectRules, Rule, TurnStats, Verdict};
use crate::{comments, copies, dependencies, integrity, protected};

pub fn judge_change(index: &Index, change: &Change, rules: &ProjectRules, exceptions: &Exceptions) -> Verdict {
    judge_since(index, change, change.before.as_deref(), rules, exceptions)
}

pub fn judge_since(index: &Index, change: &Change, approved: Option<&str>, rules: &ProjectRules, exceptions: &Exceptions) -> Verdict {
    let since_approved = Change { path: change.path.clone(), before: approved.map(str::to_string), after: change.after.clone() };
    let mut verdict = Verdict::default()
        .with(integrity::change(change))
        .with(copies::copies(index, change))
        .with(dependencies::findings(change))
        .with(protected::findings(&since_approved));
    if rules.no_comments {
        verdict = verdict.with(comments::findings(change));
    }
    exceptions.filter(verdict)
}

pub fn judge_command(command: &str) -> Verdict {
    Verdict::default().with(integrity::command(command))
}

fn pair(finding: &Finding) -> Option<(String, String)> {
    if !matches!(finding.rule, Rule::R1 | Rule::R2) {
        return None;
    }
    let (_, sides) = finding.key.split_once(':')?;
    let (left, right) = sides.split_once('~')?;
    Some(if left < right { (left.to_string(), right.to_string()) } else { (right.to_string(), left.to_string()) })
}

fn lines(before: &str, after: &str) -> (u64, u64) {
    let mut left: HashMap<&str, i64> = HashMap::new();
    for line in before.lines() {
        *left.entry(line).or_default() += 1;
    }
    let mut added = 0;
    for line in after.lines() {
        match left.get_mut(line) {
            Some(count) if *count > 0 => *count -= 1,
            _ => added += 1,
        }
    }
    (added, left.values().map(|count| *count as u64).sum())
}

fn stats(changes: &[Change]) -> TurnStats {
    let mut stats = TurnStats::default();
    for change in changes {
        let (added, removed) = lines(change.before(), change.after());
        stats.lines_added += added;
        stats.lines_removed += removed;
        stats.files_added += u64::from(change.before.is_none() && change.after.is_some());
        let known: HashSet<u64> = build::analyze(&change.path, change.before()).iter().map(|unit| unit.print.exact).collect();
        stats.units_added += build::analyze(&change.path, change.after()).iter().filter(|unit| unit.whole && !known.contains(&unit.print.exact)).count() as u64;
    }
    stats
}

pub fn judge_turn(index: &Index, changes: &[Change], dead_before: &Dead, rules: &ProjectRules, exceptions: &Exceptions) -> (Verdict, TurnStats) {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut verdict = Verdict::default();
    for change in changes {
        let found = judge_change(index, change, rules, exceptions).findings.into_iter().filter(|finding| pair(finding).is_none_or(|pair| seen.insert(pair)));
        verdict = verdict.with(found);
    }
    let verdict = verdict.with(exceptions.filter(Verdict::default().with(orphans::findings(index, dead_before))).findings);
    (verdict, stats(changes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verdict::{Outcome, Severity};

    const TOTALS: &str = "export function totals(rows: Row[], limit: number) {
  let sum = 0;
  let count = 0;
  const skipped: string[] = [];
  for (const row of rows) {
    if (!row.active || row.amount > limit) {
      skipped.push(row.id);
      continue;
    }
    sum += row.amount * row.weight;
    count += row.weight;
  }
  const mean = count > 0 ? sum / count : 0;
  return { sum, count, mean, skipped, ratio: rows.length ? count / rows.length : 0 };
}
";

    fn project(name: &str, files: &[(&str, &str)]) -> Index {
        let root = std::env::temp_dir().join("sens-canon-judge").join(name);
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        build::build(&root)
    }

    #[test]
    fn a_write_is_judged_by_every_change_rule_and_exceptions_are_honoured() {
        let index = project("change", &[("src/lib/totals.ts", TOTALS)]);
        let copy = Change { path: "src/report.ts".into(), before: None, after: Some(format!("// sums\n{}", TOTALS.replace("totals", "summarize"))) };
        let strict = ProjectRules { no_comments: true };
        let verdict = judge_change(&index, &copy, &strict, &Exceptions::default());
        let rules: Vec<Rule> = verdict.findings.iter().map(|finding| finding.rule).collect();
        assert_eq!(rules, [Rule::R1, Rule::R6]);
        assert_eq!(verdict.outcome(), Outcome::Deny);
        let accepted = Exceptions { keys: verdict.findings.iter().map(|finding| finding.key.clone()).collect() };
        assert_eq!(judge_change(&index, &copy, &strict, &accepted).outcome(), Outcome::Pass);
        assert_eq!(judge_change(&index, &copy, &ProjectRules::default(), &Exceptions::default()).findings.len(), 1);
    }

    #[test]
    fn a_turn_reports_a_copy_between_its_own_files_once_and_counts_what_it_added() {
        let first = TOTALS.replace("totals", "first");
        let second = TOTALS.replace("totals", "second").replace("rows", "items").replace("row", "item");
        let index = project("turn", &[("package.json", r#"{ "main": "src/index.ts" }"#), ("src/index.ts", "import { first } from './a.ts';\nimport { second } from './b.ts';\nfirst([], 1);\nsecond([], 1);\n"), ("src/a.ts", &first), ("src/b.ts", &second)]);
        let changes = [Change { path: "src/a.ts".into(), before: None, after: Some(first.clone()) }, Change { path: "src/b.ts".into(), before: None, after: Some(second.clone()) }];
        let (verdict, stats) = judge_turn(&index, &changes, &Dead::new(), &ProjectRules::default(), &Exceptions::default());
        let copies: Vec<&Finding> = verdict.findings.iter().filter(|finding| finding.rule == Rule::R1).collect();
        assert_eq!(copies.len(), 1, "{:?}", verdict.findings);
        assert_eq!(copies[0].severity, Severity::Block);
        assert_eq!((stats.files_added, stats.units_added), (2, 2));
        assert_eq!(stats.lines_added, (first.lines().count() + second.lines().count()) as u64);
    }

    #[test]
    fn tests_written_in_this_turn_can_be_reshaped_but_the_approved_ones_stay_protected() {
        let approved = "import { test } from 'node:test';
import assert from 'node:assert/strict';

test('adds', () => {
  assert.equal(add(1), 2);
});
";
        let grown = format!("{approved}
test('adds more', () => {{
  assert.equal(add(2), 3);
  assert.equal(add(3), 4);
}});
");
        let protects = |change: &Change| judge_since(&Index::default(), change, Some(approved), &ProjectRules::default(), &Exceptions::default()).findings.iter().any(|finding| finding.rule == Rule::R8);
        let reshaped = Change { path: "test/add.test.ts".into(), before: Some(grown.clone()), after: Some(grown.replace("  assert.equal(add(2), 3);
", "")) };
        let withdrawn = Change { path: "test/add.test.ts".into(), before: Some(grown.clone()), after: Some(approved.into()) };
        let dropped = Change { path: "test/add.test.ts".into(), before: Some(grown.clone()), after: Some("import { test } from 'node:test';
".into()) };
        assert!(!protects(&reshaped) && !protects(&withdrawn));
        assert!(protects(&dropped));
        assert!(judge_change(&Index::default(), &reshaped, &ProjectRules::default(), &Exceptions::default()).findings.iter().any(|finding| finding.rule == Rule::R8));
    }

    #[test]
    fn a_command_is_judged_for_integrity_only() {
        assert_eq!(judge_command("npm test").outcome(), Outcome::Pass);
        assert_eq!(judge_command("rm -rf .sens").outcome(), Outcome::Deny);
    }
}
