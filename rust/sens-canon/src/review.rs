use std::collections::BTreeMap;
use std::fmt::Write as _;

use sens_index::index::Index;
use sens_index::testfile::is_test_file;
use serde_json::{Value, json};

use crate::card;
use crate::relevant::Catalog;
use crate::verdict::{Finding, Rule, Severity, Target};

pub const BRIEF: &str = include_str!("review.md");
const CANDIDATES: usize = 3;
const EXCERPT_LINES: usize = 5;
const DIFF_CAP: usize = 80_000;
const KEY_CAP: usize = 80;
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

impl Review {
    pub fn of(index: &Index, catalog: &Catalog, request: &str, diff: &str) -> Review {
        let sides = sides(diff);
        let candidates = sides
            .iter()
            .filter(|(file, side)| !is_test_file(file) && !side.added.is_empty())
            .map(|(file, side)| {
                let code: String = side.added.iter().map(|(_, text)| format!("{text}\n")).collect();
                let found: Vec<Candidate> = catalog
                    .like_code(&code, file)
                    .into_iter()
                    .take(CANDIDATES)
                    .map(|suggestion| {
                        let symbol = &index.symbols[suggestion.symbol];
                        Candidate {
                            name: symbol.name.clone(),
                            file: symbol.file.clone(),
                            line: symbol.line,
                            signature: symbol.signature.clone(),
                            uses: suggestion.uses,
                            excerpt: excerpt(index, &symbol.file, symbol.line),
                        }
                    })
                    .collect();
                (file.clone(), found)
            })
            .filter(|(_, found)| !found.is_empty())
            .collect();
        Review { request: request.to_string(), diff: diff.to_string(), sides, candidates, installed: installed(index) }
    }

    pub fn prompt(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# The person's request\n\n{}\n", self.request.trim());
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
                let _ = writeln!(out, "- `{}` at {} (used {} times)\n~~~\n{}\n~~~", candidate.signature.trim(), candidate.cited(), candidate.uses, candidate.excerpt);
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
        if !whole.contains(&quote) {
            return None;
        }
        let opening = squeezed(item["quote"].as_str()?.lines().find(|line| !line.trim().is_empty())?);
        let line = lines.iter().find(|(_, text)| squeezed(text).contains(&opening)).map_or(0, |(number, _)| *number);
        let target = match rule {
            Rule::S7 => {
                let candidate = self.candidate(item["cites"].as_str()?)?;
                Some(Target { symbol: candidate.name.clone(), file: candidate.file.clone(), line: candidate.line, signature: candidate.signature.clone(), excerpt: candidate.excerpt.clone() })
            }
            _ => None,
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
        let candidate = Candidate { name: "plain".into(), file: "src/market/search.js".into(), line: 3, signature: "export const plain = (text) =>".into(), uses: 9, excerpt: "export const plain = (text) => text".into() };
        Review { request: "Find sessions without accents.".into(), diff: DIFF.into(), sides: sides(DIFF), candidates: BTreeMap::from([("src/bar.ts".to_string(), vec![candidate])]), installed: vec!["dayjs (package.json)".into()] }
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
        assert!(review.findings(&json!({ "nothing": true })).is_empty());
    }

    #[test]
    fn the_prompt_carries_the_request_the_candidates_the_dependencies_and_the_diff() {
        let prompt = review().prompt();
        for part in ["Find sessions without accents.", "dayjs (package.json)", "src/market/search.js:3 (used 9 times)", "+export interface Picker", "~~~diff"] {
            assert!(prompt.contains(part), "{part} missing in:\n{prompt}");
        }
        assert!(schema().contains("\"S7\""));
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
        let review = Review::of(&index, &catalog, "Ignore accents.", diff);
        let named: Vec<&str> = review.candidates.get("src/bar.ts").into_iter().flatten().map(|candidate| candidate.name.as_str()).collect();
        assert_eq!(named.first(), Some(&"plain"), "{named:?}");
        assert!(review.candidates["src/bar.ts"][0].excerpt.contains("normalize"));
        assert!(review.installed.iter().any(|name| name.starts_with("dayjs")), "{:?}", review.installed);
    }
}
