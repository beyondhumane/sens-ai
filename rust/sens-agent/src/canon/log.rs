use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sens_canon::verdict::{Finding, Rule, Severity};
use serde::{Deserialize, Serialize};

const FOLDER: [&str; 2] = [".sens", "canon"];
const FILE: &str = "log.jsonl";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Decision {
    Blocked,
    Considered,
    Kept,
    Allowed,
    Refused,
    Noted,
    Held,
    Unjudged,
    Accepted,
    Undone,
    Reviewed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Entry {
    pub at: u64,
    pub session: String,
    pub turn: u64,
    pub canon: String,
    pub stage: String,
    pub rule: Option<Rule>,
    pub severity: Option<Severity>,
    pub file: String,
    pub line: u32,
    pub key: String,
    pub target: Option<String>,
    pub round: u32,
    pub decision: Option<Decision>,
    pub cost: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Avoided {
    pub copies: u32,
    pub comments: u32,
    pub dependencies: u32,
    pub protected: u32,
    pub tests: u32,
    pub orphans: u32,
    pub judgment: u32,
    pub held: u32,
    pub accepted: u32,
    pub reviews: u32,
    pub reviewer_cost: f64,
}

fn file(work: &Path) -> PathBuf {
    FOLDER.iter().fold(work.to_path_buf(), |path, part| path.join(part)).join(FILE)
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_millis() as u64)
}

pub fn entry(finding: &Finding, decision: Decision) -> Entry {
    Entry {
        at: now(),
        rule: Some(finding.rule),
        severity: Some(finding.severity),
        file: finding.file.clone(),
        line: finding.line,
        key: finding.key.clone(),
        target: finding.target.as_ref().map(|target| format!("{}:{} {}", target.file, target.line, target.symbol)),
        decision: Some(decision),
        ..Entry::default()
    }
}

pub fn write(work: &Path, entries: &[Entry]) -> Result<(), String> {
    if entries.is_empty() {
        return Ok(());
    }
    let target = file(work);
    let folder = target.parent().unwrap_or(work);
    std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    let mut log = std::fs::OpenOptions::new().create(true).append(true).open(&target).map_err(|error| format!("{}: {error}", target.display()))?;
    let lines: String = entries.iter().filter_map(|entry| serde_json::to_string(entry).ok()).map(|line| format!("{line}\n")).collect();
    log.write_all(lines.as_bytes()).map_err(|error| format!("{}: {error}", target.display()))
}

pub fn read(work: &Path) -> Vec<Entry> {
    std::fs::read_to_string(file(work)).unwrap_or_default().lines().filter_map(|line| serde_json::from_str(line).ok()).collect()
}

fn stopped(decision: Option<Decision>) -> bool {
    matches!(decision, Some(Decision::Blocked | Decision::Considered | Decision::Refused))
}

pub fn avoided(work: &Path, since: u64) -> Avoided {
    let entries: Vec<Entry> = read(work).into_iter().filter(|entry| entry.at >= since).collect();
    let given_up: BTreeSet<&str> = entries.iter().filter(|entry| matches!(entry.decision, Some(Decision::Kept | Decision::Accepted))).map(|entry| entry.key.as_str()).collect();
    let mut stops: BTreeMap<&str, Rule> = BTreeMap::new();
    for entry in entries.iter().filter(|entry| stopped(entry.decision) && !given_up.contains(entry.key.as_str())) {
        if let Some(rule) = entry.rule {
            stops.insert(entry.key.as_str(), rule);
        }
    }
    let mut avoided = Avoided::default();
    for rule in stops.values() {
        let slot = match rule {
            Rule::R1 | Rule::R2 => &mut avoided.copies,
            Rule::R3 => &mut avoided.dependencies,
            Rule::R4 => &mut avoided.orphans,
            Rule::R6 => &mut avoided.comments,
            Rule::R5 | Rule::R7 => &mut avoided.protected,
            Rule::R8 => &mut avoided.tests,
            _ => &mut avoided.judgment,
        };
        *slot += 1;
    }
    let turns = |decision: Decision| entries.iter().filter(|entry| entry.decision == Some(decision)).map(|entry| (entry.session.as_str(), entry.turn)).collect::<BTreeSet<_>>().len() as u32;
    avoided.held = turns(Decision::Held) + turns(Decision::Unjudged);
    avoided.accepted = turns(Decision::Accepted);
    let reviews: Vec<&Entry> = entries.iter().filter(|entry| entry.decision == Some(Decision::Reviewed)).collect();
    avoided.reviews = reviews.len() as u32;
    avoided.reviewer_cost = reviews.iter().map(|entry| entry.cost).sum();
    avoided
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(rule: Rule, key: &str) -> Finding {
        Finding { rule, severity: Severity::Block, file: "src/a.ts".into(), line: 3, message: String::new(), target: None, key: key.into() }
    }

    fn at(entry: Entry, turn: u64) -> Entry {
        Entry { session: "s1".into(), turn, ..entry }
    }

    #[test]
    fn what_was_stopped_and_stayed_stopped_counts_as_avoided() {
        let work = std::env::temp_dir().join("sens-canon-log");
        let _ = std::fs::remove_dir_all(&work);
        let copy = finding(Rule::R1, "R1:a");
        write(&work, &[
            at(entry(&copy, Decision::Blocked), 1),
            at(entry(&copy, Decision::Blocked), 1),
            at(entry(&finding(Rule::R2, "R2:b"), Decision::Considered), 1),
            at(entry(&finding(Rule::R2, "R2:b"), Decision::Kept), 1),
            at(entry(&finding(Rule::R3, "R3:moment"), Decision::Refused), 2),
            at(entry(&finding(Rule::R3, "R3:dayjs"), Decision::Allowed), 2),
            at(entry(&finding(Rule::S1, "S1:c"), Decision::Blocked), 3),
            at(entry(&finding(Rule::R6, "R6:d"), Decision::Held), 3),
            at(entry(&finding(Rule::R6, "R6:d"), Decision::Accepted), 3),
            at(Entry { decision: Some(Decision::Reviewed), cost: 0.004, ..Entry::default() }, 3),
            at(Entry { decision: Some(Decision::Reviewed), cost: 0.006, ..Entry::default() }, 4),
        ])
        .unwrap();
        let avoided = avoided(&work, 0);
        assert_eq!((avoided.copies, avoided.dependencies, avoided.judgment, avoided.comments), (1, 1, 1, 0));
        assert_eq!((avoided.held, avoided.accepted, avoided.reviews), (1, 1, 2));
        assert!((avoided.reviewer_cost - 0.01).abs() < 1e-9);
        assert_eq!(read(&work).len(), 11);
        assert_eq!(self::avoided(&work, now() + 1_000), Avoided::default());
    }
}
