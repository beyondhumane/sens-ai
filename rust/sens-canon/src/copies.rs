use std::collections::HashSet;

use sens_index::build;
use sens_index::fingerprint::{Likeness, NEAR, Unit, resemblance};
use sens_index::index::Index;

use crate::verdict::{Change, Finding, Rule, Severity, Target};

pub const BLOCKING_TOKENS: usize = 80;
const EXCERPT_LINES: usize = 5;

struct Candidate<'a> {
    unit: &'a Unit,
    likeness: Likeness,
    source: Option<&'a str>,
}

fn rank(likeness: Likeness) -> f32 {
    match likeness {
        Likeness::Exact => 3.0,
        Likeness::Renamed => 2.0,
        Likeness::Near(share) => share,
    }
}

fn likeness(unit: &Unit, other: &Unit) -> Option<Likeness> {
    if unit.print.exact == other.print.exact {
        return Some(Likeness::Exact);
    }
    if unit.print.renamed == other.print.renamed {
        return Some(Likeness::Renamed);
    }
    let share = resemblance(&unit.print.shingles, &other.print.shingles);
    (share >= NEAR).then_some(Likeness::Near(share))
}

fn best<'a>(index: &'a Index, path: &str, unit: &Unit, siblings: &'a [Unit], text: &'a str) -> Option<Candidate<'a>> {
    let indexed = index
        .similar(&unit.print)
        .into_iter()
        .map(|found| Candidate { unit: &index.units[found.unit], likeness: found.likeness, source: None })
        .filter(|candidate| candidate.unit.file != path);
    let nearby = siblings
        .iter()
        .filter(|other| other.symbol != unit.symbol && other.comparable())
        .filter_map(|other| likeness(unit, other).map(|likeness| Candidate { unit: other, likeness, source: Some(text) }));
    indexed
        .chain(nearby)
        .filter(|candidate| candidate.unit.test == unit.test)
        .max_by(|a, b| rank(a.likeness).total_cmp(&rank(b.likeness)).then_with(|| b.unit.symbol.cmp(&a.unit.symbol)))
}

fn excerpt(index: &Index, candidate: &Candidate) -> String {
    let text = match candidate.source {
        Some(text) => text.to_string(),
        None => std::fs::read_to_string(index.root.join(&candidate.unit.file)).unwrap_or_default(),
    };
    text.lines().skip(candidate.unit.start_line.saturating_sub(1) as usize).take(EXCERPT_LINES).collect::<Vec<_>>().join("\n")
}

fn message(path: &str, unit: &Unit, target: &Unit, likeness: Likeness) -> String {
    let there = format!("`{}` ({}:{})", target.name, target.file, target.start_line);
    match (unit.whole, likeness) {
        (true, Likeness::Near(share)) => format!(
            "`{}` ({path}:{}) is a near copy of {there}, {:.0}% the same. Reuse or extend `{}` instead of keeping two versions.",
            unit.name,
            unit.start_line,
            share * 100.0,
            target.name
        ),
        (true, _) => format!(
            "`{}` ({path}:{}) repeats {there}. Call `{}` instead of writing it again; if it needs a small change, make it there so both callers share it.",
            unit.name, unit.start_line, target.name
        ),
        (false, _) => format!(
            "Lines {}-{} of `{}` in {path} repeat code in {there}. Extract the shared part once and call it from both places.",
            unit.start_line, unit.end_line, unit.name
        ),
    }
}

pub fn copies(index: &Index, change: &Change) -> Vec<Finding> {
    let Some(after) = change.after.as_deref() else {
        return Vec::new();
    };
    let before_units = build::analyze(&change.path, change.before());
    let after_units = build::analyze(&change.path, after);
    let known: HashSet<u64> = before_units.iter().map(|unit| unit.print.exact).collect();
    let mut reported: HashSet<(String, String)> = HashSet::new();
    let mut findings = Vec::new();
    let mut fresh: Vec<&Unit> = after_units.iter().filter(|unit| unit.comparable() && !known.contains(&unit.print.exact)).collect();
    fresh.sort_by_key(|unit| !unit.whole);
    for unit in fresh {
        let Some(found) = best(index, &change.path, unit, &after_units, after) else {
            continue;
        };
        let target = found.unit;
        if !reported.insert((unit.symbol.clone(), target.symbol.clone())) {
            continue;
        }
        let inherited = before_units
            .iter()
            .filter(|old| old.name == unit.name && old.comparable())
            .any(|old| best(index, &change.path, old, &before_units, change.before()).is_some_and(|earlier| earlier.unit.symbol == target.symbol || earlier.unit.name == target.name));
        if inherited {
            continue;
        }
        let rule = match found.likeness {
            Likeness::Near(_) => Rule::R2,
            _ => Rule::R1,
        };
        let severity = if !unit.test && unit.print.tokens.min(target.print.tokens) >= BLOCKING_TOKENS { Severity::Block } else { Severity::Note };
        let text = excerpt(index, &found);
        findings.push(Finding {
            rule,
            severity,
            file: change.path.clone(),
            line: unit.start_line,
            message: message(&change.path, unit, target, found.likeness),
            target: Some(Target {
                symbol: target.symbol.clone(),
                file: target.file.clone(),
                line: target.start_line,
                signature: text.lines().next().unwrap_or_default().trim().to_string(),
                excerpt: text,
            }),
            key: format!("{rule:?}:{}:{}~{}:{}", change.path, unit.name, target.file, target.name),
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

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

    const SMALL: &str = "export function label(user: User) {
  const first = user.first.trim();
  const last = user.last.trim();
  return [first, last].filter(Boolean).join(' ');
}
";

    fn project(name: &str, files: &[(&str, &str)]) -> Index {
        let root = std::env::temp_dir().join("sens-canon-copies").join(name);
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        build::build(&root)
    }

    fn new_file(path: &str, content: &str) -> Change {
        Change { path: path.into(), before: None, after: Some(content.into()) }
    }

    fn renamed(source: &str, from: &str, to: &str) -> String {
        source.replace(from, to).replace("rows", "entries").replace("row", "entry").replace("sum", "total")
    }

    #[test]
    fn a_renamed_copy_of_a_large_function_is_blocked_with_its_target() {
        let index = project("renamed", &[("src/lib/totals.ts", TOTALS)]);
        let found = copies(&index, &new_file("src/report.ts", &renamed(TOTALS, "totals", "summarize")));
        assert_eq!(found.len(), 1, "{found:?}");
        let finding = &found[0];
        assert_eq!((finding.rule, finding.severity), (Rule::R1, Severity::Block));
        let target = finding.target.as_ref().unwrap();
        assert_eq!((target.file.as_str(), target.line), ("src/lib/totals.ts", 1));
        assert!(target.excerpt.starts_with("export function totals("));
        assert!(finding.message.contains("Call `totals`"), "{}", finding.message);
    }

    #[test]
    fn a_copy_of_a_small_function_is_only_noted() {
        let index = project("small", &[("src/names.ts", SMALL)]);
        let found = copies(&index, &new_file("src/card.ts", &SMALL.replace("label", "fullName").replace("user", "person")));
        assert_eq!(found.iter().map(|finding| (finding.rule, finding.severity)).collect::<Vec<_>>(), [(Rule::R1, Severity::Note)]);
    }

    #[test]
    fn a_near_copy_is_r2_and_names_how_much_it_shares() {
        let index = project("near", &[("src/lib/totals.ts", TOTALS)]);
        let near = TOTALS.replace("totals", "tally").replace("  const mean", "  audit(rows);\n  const mean");
        let found = copies(&index, &new_file("src/tally.ts", &near));
        assert_eq!(found[0].rule, Rule::R2);
        assert_eq!(found[0].severity, Severity::Block);
        assert!(found[0].message.contains("% the same"), "{}", found[0].message);
    }

    #[test]
    fn two_new_copies_in_the_same_file_are_caught() {
        let index = project("siblings", &[("src/other.ts", SMALL)]);
        let both = format!("{TOTALS}\n{}", renamed(TOTALS, "totals", "again"));
        let found = copies(&index, &new_file("src/double.ts", &both));
        assert!(found.iter().any(|finding| finding.rule == Rule::R1 && finding.target.as_ref().is_some_and(|target| target.file == "src/double.ts")), "{found:?}");
    }

    #[test]
    fn a_copy_that_was_already_there_is_not_blamed_on_the_turn() {
        let copied = renamed(TOTALS, "totals", "summarize");
        let before = format!("{copied}\nexport const version = 1;\n");
        let after = format!("{copied}\nexport const version = 2;\n");
        let index = project("debt", &[("src/lib/totals.ts", TOTALS), ("src/report.ts", &before)]);
        let change = Change { path: "src/report.ts".into(), before: Some(before), after: Some(after) };
        assert!(copies(&index, &change).is_empty());
    }

    #[test]
    fn copies_between_tests_are_notes_and_strangers_are_left_alone() {
        let index = project("tests", &[("src/lib/totals.ts", TOTALS), ("test/totals.test.ts", TOTALS)]);
        let in_test = copies(&index, &new_file("test/again.test.ts", &renamed(TOTALS, "totals", "again")));
        assert!(in_test.iter().all(|finding| finding.severity == Severity::Note), "{in_test:?}");
        assert!(copies(&index, &new_file("src/names.ts", SMALL)).is_empty());
    }

    #[test]
    fn a_deleted_file_has_nothing_to_copy() {
        let index = project("deleted", &[("src/lib/totals.ts", TOTALS)]);
        assert!(copies(&index, &Change { path: "src/lib/totals.ts".into(), before: Some(TOTALS.into()), after: None }).is_empty());
    }
}
