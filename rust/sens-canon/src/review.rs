use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::LazyLock;

use regex::Regex;

use sens_index::build;
use sens_index::index::Index;
use sens_index::testfile::is_test_file;
use serde_json::{Value, json};

use crate::card;
use crate::relevant::Catalog;
use crate::verdict::{Finding, Rule, Severity, Target};

pub const BRIEF: &str = include_str!("review.md");
const CANDIDATES: usize = 5;
const EXCERPT_LINES: usize = 5;
const DIFF_CAP: usize = 80_000;
const KEY_CAP: usize = 80;
static RAISES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bthrow\b|\braise\b|panic!|\bbail!|\bErr\(").expect("a valid pattern"));

const RULES: [(&str, Rule); 7] = [("S1", Rule::S1), ("S2", Rule::S2), ("S3", Rule::S3), ("S4", Rule::S4), ("S5", Rule::S5), ("S6", Rule::S6), ("S7", Rule::S7)];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sides {
    pub added: Vec<(u32, String)>,
    pub removed: Vec<(u32, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub signature: String,
    pub uses: usize,
    pub exported: bool,
    pub excerpt: String,
}

impl Candidate {
    fn cited(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Review {
    pub request: String,
    pub diff: String,
    pub sides: BTreeMap<String, Sides>,
    pub candidates: BTreeMap<String, Vec<Candidate>>,
    pub installed: Vec<String>,
}

pub fn schema() -> String {
    let rules: Vec<&str> = RULES.iter().map(|(name, _)| *name).collect();
    json!({
        "type": "object",
        "properties": {
            "findings": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "rule": { "type": "string", "enum": rules },
                        "file": { "type": "string" },
                        "quote": { "type": "string" },
                        "why": { "type": "string" },
                        "fix": { "type": "string" },
                        "confidence": { "type": "string", "enum": ["high", "medium"] },
                        "cites": { "type": "string" }
                    },
                    "required": ["rule", "file", "quote", "why", "fix", "confidence"]
                }
            }
        },
        "required": ["findings"]
    })
    .to_string()
}

fn hunk_starts(line: &str) -> Option<(u32, u32)> {
    let mut parts = line.strip_prefix("@@ ")?.split_whitespace();
    let old = parts.next()?.strip_prefix('-')?.split(',').next()?.parse().ok()?;
    let new = parts.next()?.strip_prefix('+')?.split(',').next()?.parse().ok()?;
    Some((old, new))
}

pub fn sides(diff: &str) -> BTreeMap<String, Sides> {
    let mut found: BTreeMap<String, Sides> = BTreeMap::new();
    let (mut from, mut to) = (String::new(), String::new());
    let (mut old, mut new) = (0u32, 0u32);
    let mut in_hunk = false;
    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            in_hunk = false;
            (from, to) = (String::new(), String::new());
            continue;
        }
        if !in_hunk {
            if let Some(path) = line.strip_prefix("--- a/") {
                from = path.to_string();
            } else if let Some(path) = line.strip_prefix("+++ b/") {
                to = path.to_string();
            }
        }
        if let Some((first_old, first_new)) = hunk_starts(line) {
            (old, new, in_hunk) = (first_old, first_new, true);
            continue;
        }
        if !in_hunk {
            continue;
        }
        let file = if to.is_empty() { from.clone() } else { to.clone() };
        let sides = found.entry(file).or_default();
        match line.chars().next() {
            Some('+') => {
                sides.added.push((new, line[1..].to_string()));
                new += 1;
            }
            Some('-') => {
                sides.removed.push((old, line[1..].to_string()));
                old += 1;
            }
            Some('\\') => {}
            _ => {
                old += 1;
                new += 1;
            }
        }
    }
    found
}

fn squeezed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn excerpt(index: &Index, file: &str, line: u32) -> String {
    let text = std::fs::read_to_string(index.root.join(file)).unwrap_or_default();
    text.lines().skip(line.saturating_sub(1) as usize).take(EXCERPT_LINES).collect::<Vec<_>>().join("\n")
}

fn installed(index: &Index) -> Vec<String> {
    card::manifests(index).into_iter().flat_map(|(manifest, names)| names.into_iter().map(move |name| format!("{name} ({manifest})"))).collect()
}

fn pieces(index: &Index, file: &str, side: &Sides) -> Vec<String> {
    let text = std::fs::read_to_string(index.root.join(file)).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let added: BTreeSet<u32> = side.added.iter().map(|(line, _)| *line).collect();
    let said = |numbers: &BTreeSet<u32>| numbers.iter().filter_map(|line| lines.get(*line as usize - 1)).map(|line| format!("{line}
")).collect::<String>();
    let mut covered: BTreeSet<u32> = BTreeSet::new();
    let mut found: Vec<String> = Vec::new();
    for unit in build::analyze(file, &text).iter().filter(|unit| unit.whole) {
        let inside: BTreeSet<u32> = added.range(unit.start_line..=unit.end_line).copied().collect();
        if !inside.is_empty() {
            found.push(said(&inside));
            covered.extend(inside);
        }
    }
    let rest: BTreeSet<u32> = added.difference(&covered).copied().collect();
    if !rest.is_empty() {
        found.push(said(&rest));
    }
    if found.iter().all(|piece| piece.trim().is_empty()) {
        return vec![side.added.iter().map(|(_, text)| format!("{text}
")).collect()];
    }
    found
}

fn candidates(index: &Index, catalog: &Catalog, file: &str, side: &Sides) -> Vec<Candidate> {
    let mut best: BTreeMap<usize, f32> = BTreeMap::new();
    for piece in pieces(index, file, side) {
        for suggestion in catalog.like_code(&piece, file) {
            let score = best.entry(suggestion.symbol).or_default();
            *score = score.max(suggestion.score);
        }
    }
    let mut ranked: Vec<(usize, f32)> = best.into_iter().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(CANDIDATES)
        .map(|(at, _)| {
            let symbol = &index.symbols[at];
            Candidate { name: symbol.name.clone(), file: symbol.file.clone(), line: symbol.line, signature: symbol.signature.clone(), uses: index.raw_references(at).len(), exported: symbol.exported, excerpt: excerpt(index, &symbol.file, symbol.line) }
        })
        .collect()
}

impl Review {
    pub fn of(index: &Index, catalog: &Catalog, request: &str, diff: &str) -> Review {
        let sides = sides(diff);
        let candidates = sides
            .iter()
            .filter(|(file, side)| !is_test_file(file) && !side.added.is_empty())
            .map(|(file, side)| (file.clone(), candidates(index, catalog, file, side)))
            .filter(|(_, found)| !found.is_empty())
            .collect();
        Review { request: request.to_string(), diff: diff.to_string(), sides, candidates, installed: installed(index) }
    }

    pub fn prompt(&self, tongue: &str) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# The person's request\n\n{}\n", self.request.trim());
        let _ = writeln!(out, "# Language\n\nThe person reads your findings: write `why` and `fix` in {tongue}. Copy `quote` from the diff exactly as it is.\n");
        let _ = writeln!(out, "# Installed dependencies\n");
        if self.installed.is_empty() {
            let _ = writeln!(out, "None.");
        }
        for name in &self.installed {
            let _ = writeln!(out, "- {name}");
        }
        let _ = writeln!(out, "\n# Candidates for S7\n");
        if self.candidates.is_empty() {
            let _ = writeln!(out, "None.");
        }
        for (file, found) in &self.candidates {
            let _ = writeln!(out, "For {file}:\n");
            for candidate in found {
                let reach = if candidate.exported { "exported" } else { "private to its file" };
                let _ = writeln!(out, "- `{}` at {} ({reach}, used {} times)\n~~~\n{}\n~~~", candidate.signature.trim(), candidate.cited(), candidate.uses, candidate.excerpt);
            }
            let _ = writeln!(out);
        }
        let diff = match self.diff.char_indices().nth(DIFF_CAP) {
            Some((cut, _)) => format!("{}\n[the rest of the diff is left out]", &self.diff[..cut]),
            None => self.diff.clone(),
        };
        let _ = writeln!(out, "# The diff of the turn\n\n~~~diff\n{}\n~~~", diff.trim_end());
        out
    }

    fn candidate(&self, cited: &str) -> Option<&Candidate> {
        self.candidates.values().flatten().find(|candidate| candidate.cited() == cited.trim().trim_matches('`'))
    }

    pub fn findings(&self, answer: &Value) -> Vec<Finding> {
        answer["findings"].as_array().into_iter().flatten().filter_map(|item| self.finding(item)).collect()
    }

    fn finding(&self, item: &Value) -> Option<Finding> {
        let rule = RULES.iter().find(|(name, _)| item["rule"].as_str() == Some(name)).map(|(_, rule)| *rule)?;
        let file = item["file"].as_str()?.trim().trim_start_matches("./");
        let severity = match item["confidence"].as_str()? {
            "high" => Severity::Block,
            "medium" => Severity::Note,
            _ => return None,
        };
        let quote = squeezed(item["quote"].as_str()?);
        if quote.is_empty() {
            return None;
        }
        let side = self.sides.get(file)?;
        let lines = if rule == Rule::S6 { &side.removed } else { &side.added };
        let whole = squeezed(&lines.iter().map(|(_, text)| text.as_str()).collect::<Vec<_>>().join("\n"));
        if !whole.contains(&quote) || (rule == Rule::S4 && RAISES.is_match(&quote)) {
            return None;
        }
        let opening = squeezed(item["quote"].as_str()?.lines().find(|line| !line.trim().is_empty())?);
        let line = lines.iter().find(|(_, text)| squeezed(text).contains(&opening)).map_or(0, |(number, _)| *number);
        let (target, severity) = match rule {
            Rule::S7 => {
                let candidate = self.candidate(item["cites"].as_str()?)?;
                let severity = if candidate.exported { severity } else { Severity::Note };
                (Some(Target { symbol: candidate.name.clone(), file: candidate.file.clone(), line: candidate.line, signature: candidate.signature.clone(), excerpt: candidate.excerpt.clone() }), severity)
            }
            _ => (None, severity),
        };
        let said = |field: &str| item[field].as_str().unwrap_or_default().trim().to_string();
        Some(Finding {
            rule,
            severity,
            file: file.to_string(),
            line,
            message: format!("{} {}", said("why"), said("fix")).trim().to_string(),
            target,
            key: format!("{rule:?}:{file}:{}", quote.chars().take(KEY_CAP).collect::<String>()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::build;

    const DIFF: &str = "diff --git a/src/bar.ts b/src/bar.ts
index 1111111..2222222 100644
--- a/src/bar.ts
+++ b/src/bar.ts
@@ -1,4 +1,5 @@
 import { stem } from './format';
-const lower = (text: string) => text.toLocaleLowerCase();
+const lower = (text: string) => text.normalize(\"NFD\").replace(/[\\u0300-\\u036f]/g, \"\").toLocaleLowerCase();
+export interface Picker { pick(all: string[]): string[] }
 export const pick = (all: string[]) => all;
--- keep this line
diff --git a/src/gone.ts b/src/gone.ts
deleted file mode 100644
--- a/src/gone.ts
+++ /dev/null
@@ -1,2 +0,0 @@
-if (!allowed(user)) throw new Error('no');
-save(user);
";

    fn review() -> Review {
        let candidate = Candidate { name: "plain".into(), file: "src/market/search.js".into(), line: 3, signature: "export const plain = (text) =>".into(), uses: 9, exported: true, excerpt: "export const plain = (text) => text".into() };
        let private = Candidate { name: "folded".into(), file: "src/clip.ts".into(), line: 7, signature: "const folded = (text) =>".into(), uses: 2, exported: false, excerpt: "const folded = (text) => text".into() };
        Review { request: "Find sessions without accents.".into(), diff: DIFF.into(), sides: sides(DIFF), candidates: BTreeMap::from([("src/bar.ts".to_string(), vec![candidate, private])]), installed: vec!["dayjs (package.json)".into()] }
    }

    #[test]
    fn a_diff_splits_into_added_and_removed_lines_with_their_numbers() {
        let found = sides(DIFF);
        let bar = &found["src/bar.ts"];
        assert_eq!(bar.added.iter().map(|(line, _)| *line).collect::<Vec<_>>(), [2, 3]);
        assert_eq!(bar.removed, [(2, "const lower = (text: string) => text.toLocaleLowerCase();".to_string()), (4, "-- keep this line".to_string())]);
        assert_eq!(found["src/gone.ts"].removed.len(), 2);
        assert!(found["src/gone.ts"].added.is_empty());
    }

    #[test]
    fn only_findings_that_quote_the_diff_survive_and_confidence_sets_how_hard_they_stop() {
        let review = review();
        let answer = json!({ "findings": [
            { "rule": "S1", "file": "src/bar.ts", "quote": "export interface  Picker { pick(all: string[]): string[] }", "why": "One use.", "fix": "Drop it.", "confidence": "high" },
            { "rule": "S4", "file": "src/bar.ts", "quote": "const invented = 1;", "why": "x", "fix": "y", "confidence": "high" },
            { "rule": "S9", "file": "src/bar.ts", "quote": "export interface", "why": "x", "fix": "y", "confidence": "high" },
            { "rule": "S5", "file": "src/bar.ts", "quote": "toLocaleLowerCase()", "why": "x", "fix": "y", "confidence": "medium" },
            { "rule": "S6", "file": "src/gone.ts", "quote": "if (!allowed(user)) throw new Error('no');", "why": "The check is gone.", "fix": "Keep it.", "confidence": "high" },
            { "rule": "S6", "file": "src/bar.ts", "quote": "export interface", "why": "x", "fix": "y", "confidence": "high" }
        ]});
        let found = review.findings(&answer);
        let seen: Vec<(Rule, Severity, &str, u32)> = found.iter().map(|finding| (finding.rule, finding.severity, finding.file.as_str(), finding.line)).collect();
        assert_eq!(seen, [(Rule::S1, Severity::Block, "src/bar.ts", 3), (Rule::S5, Severity::Note, "src/bar.ts", 2), (Rule::S6, Severity::Block, "src/gone.ts", 1)]);
        assert_eq!(found[0].message, "One use. Drop it.");
        assert!(found[0].key.starts_with("S1:src/bar.ts:export interface Picker"));
    }

    #[test]
    fn a_reinvention_must_cite_one_of_the_candidates_sens_gave() {
        let review = review();
        let quote = "text.normalize(\"NFD\")";
        let cited = review.findings(&json!({ "findings": [{ "rule": "S7", "file": "src/bar.ts", "quote": quote, "why": "plain does this.", "fix": "Import plain.", "confidence": "high", "cites": "src/market/search.js:3" }] }));
        assert_eq!(cited.len(), 1);
        assert_eq!(cited[0].target.as_ref().map(|target| target.symbol.as_str()), Some("plain"));
        let invented = json!({ "findings": [
            { "rule": "S7", "file": "src/bar.ts", "quote": quote, "why": "x", "fix": "y", "confidence": "high", "cites": "src/other.js:1" },
            { "rule": "S7", "file": "src/bar.ts", "quote": quote, "why": "x", "fix": "y", "confidence": "high" }
        ]});
        assert!(review.findings(&invented).is_empty());
        let private = review.findings(&json!({ "findings": [{ "rule": "S7", "file": "src/bar.ts", "quote": quote, "why": "folded does this.", "fix": "Use folded.", "confidence": "high", "cites": "src/clip.ts:7" }] }));
        assert_eq!(private[0].severity, Severity::Note);
        assert!(review.findings(&json!({ "nothing": true })).is_empty());
    }

    #[test]
    fn the_prompt_carries_the_request_the_candidates_the_dependencies_and_the_diff() {
        let prompt = review().prompt("Spanish as spoken in Spain");
        for part in ["Find sessions without accents.", "write `why` and `fix` in Spanish as spoken in Spain","dayjs (package.json)", "src/market/search.js:3 (exported, used 9 times)", "src/clip.ts:7 (private to its file", "+export interface Picker", "~~~diff"] {
            assert!(prompt.contains(part), "{part} missing in:\n{prompt}");
        }
        assert!(schema().contains("\"S7\""));
    }

    #[test]
    fn a_hand_made_helper_brings_the_one_the_project_uses_even_among_other_new_lines() {
        let root = std::env::temp_dir().join("sens-canon-review-size");
        let _ = std::fs::remove_dir_all(&root);
        let weigh = "export const weigh = (bytes) => {
  if (bytes < 1024) return units.bytes(String(bytes));
  if (bytes < 1024 * 1024) return units.kilobytes(String(Math.round(bytes / 1024)));
  return units.megabytes(tenths(bytes / 1024 / 1024));
};
";
        let shelf = "export const sizeOf = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
};
export const card = (item: Item) => `${item.project} · ${sizeOf(item.bytes)} · ${item.name} · ${item.kind}`;
";
        for (path, content) in [
            ("src/shared/format.js", weigh),
            ("src/clip.ts", "import { weigh } from './shared/format.js';
export const meta = (file) => weigh(file.bytes);
"),
            ("src/detail.ts", "import { weigh } from './shared/format.js';
export const row = (entry) => weigh(entry.size);
"),
            ("src/items.ts", "export const kindOf = (item: Item) => item.kind;
export const nameOf = (item: Item) => item.name;
export const projectOf = (item: Item) => item.project;
"),
            ("src/shelf.ts", shelf),
        ] {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        let index = build::build(&root);
        let catalog = Catalog::of(&index);
        let added: String = shelf.lines().map(|line| format!("+{line}
")).collect();
        let diff = format!("diff --git a/src/shelf.ts b/src/shelf.ts
--- /dev/null
+++ b/src/shelf.ts
@@ -0,0 +1,6 @@
{added}");
        let review = Review::of(&index, &catalog, "Show the size.", &diff);
        let named: Vec<&str> = review.candidates.get("src/shelf.ts").into_iter().flatten().map(|candidate| candidate.name.as_str()).collect();
        assert!(named.contains(&"weigh"), "{named:?}");
    }

    #[test]
    fn a_check_that_stops_bad_input_is_never_speculation() {
        let diff = "diff --git a/src/add.ts b/src/add.ts\n--- a/src/add.ts\n+++ b/src/add.ts\n@@ -1 +1,3 @@\n+export const add = (category: string, limit = 10) => {\n+  if (category.includes(\",\")) throw new Error(`Categoría no válida: ${category}`);\n+};\n";
        let index = Index::default();
        let review = Review::of(&index, &Catalog::of(&index), "Add expenses.", diff);
        let found = review.findings(&json!({ "findings": [
            { "rule": "S4", "file": "src/add.ts", "quote": "if (category.includes(\",\")) throw new Error(`Categoría no válida: ${category}`);", "why": "x", "fix": "y", "confidence": "high" },
            { "rule": "S4", "file": "src/add.ts", "quote": "export const add = (category: string, limit = 10) => {", "why": "x", "fix": "y", "confidence": "high" }
        ] }));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].key.contains("limit = 10"), "{found:?}");
    }

    #[test]
    fn the_brief_keeps_named_functions_input_checks_and_unseen_code_out_of_the_findings() {
        assert!(BRIEF.contains("A plain function that gives a name to a piece of logic is not an abstraction"));
        assert!(BRIEF.contains("keeps bad input from breaking the data or the file format"));
        assert!(BRIEF.contains("You see only the diff. The rest of the project exists"));
    }

    #[test]
    fn candidates_come_from_what_the_new_lines_resemble_elsewhere() {
        let root = std::env::temp_dir().join("sens-canon-review");
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in [
            ("package.json", r#"{ "dependencies": { "dayjs": "1" } }"#),
            ("src/market/search.js", r#"export const plain = (text) => text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();"#),
            ("src/market/use.js", "import { plain } from './search.js';\nexport const matching = (items, query) => items.filter((item) => plain(item.name).includes(plain(query)));\n"),
            ("src/bar.ts", "export const pick = (all: string[]) => all;\n"),
        ] {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        let index = build::build(&root);
        let catalog = Catalog::of(&index);
        let diff = "diff --git a/src/bar.ts b/src/bar.ts\n--- a/src/bar.ts\n+++ b/src/bar.ts\n@@ -1 +1,2 @@\n+const lower = (text: string) => text.normalize(\"NFD\").replace(/\\p{M}/gu, \"\").toLocaleLowerCase();\n export const pick = (all: string[]) => all;\n";
        std::fs::write(root.join("src/bar.ts"), "const lower = (text: string) => text.normalize(\"NFD\").replace(/\\p{M}/gu, \"\").toLocaleLowerCase();\nexport const pick = (all: string[]) => all;\n").unwrap();
        let review = Review::of(&index, &catalog, "Ignore accents.", diff);
        let named: Vec<&str> = review.candidates.get("src/bar.ts").into_iter().flatten().map(|candidate| candidate.name.as_str()).collect();
        assert_eq!(named.first(), Some(&"plain"), "{named:?}");
        assert!(review.candidates["src/bar.ts"][0].excerpt.contains("normalize"));
        assert!(review.installed.iter().any(|name| name.starts_with("dayjs")), "{:?}", review.installed);
    }
}
