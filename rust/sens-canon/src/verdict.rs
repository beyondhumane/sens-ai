use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rule {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Block,
    Ask,
    Note,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub symbol: String,
    pub file: String,
    pub line: u32,
    pub signature: String,
    pub excerpt: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub rule: Rule,
    pub severity: Severity,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub target: Option<Target>,
    pub key: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Verdict {
    pub findings: Vec<Finding>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Ask,
    Deny,
}

impl Verdict {
    pub fn outcome(&self) -> Outcome {
        let worst = self.findings.iter().map(|finding| finding.severity).min();
        match worst {
            Some(Severity::Block) => Outcome::Deny,
            Some(Severity::Ask) => Outcome::Ask,
            _ => Outcome::Pass,
        }
    }

    pub fn with(mut self, findings: impl IntoIterator<Item = Finding>) -> Verdict {
        self.findings.extend(findings);
        self
    }

    pub fn of(&self, severity: Severity) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(move |finding| finding.severity == severity)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

impl Change {
    pub fn before(&self) -> &str {
        self.before.as_deref().unwrap_or("")
    }

    pub fn after(&self) -> &str {
        self.after.as_deref().unwrap_or("")
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectRules {
    pub no_comments: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Exceptions {
    pub keys: BTreeSet<String>,
}

impl Exceptions {
    pub fn covers(&self, finding: &Finding) -> bool {
        finding.rule != Rule::R7 && self.keys.contains(&finding.key)
    }

    pub fn filter(&self, verdict: Verdict) -> Verdict {
        Verdict { findings: verdict.findings.into_iter().filter(|finding| !self.covers(finding)).collect() }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TurnStats {
    pub lines_added: u64,
    pub lines_removed: u64,
    pub files_added: u64,
    pub units_added: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(rule: Rule, severity: Severity, key: &str) -> Finding {
        Finding { rule, severity, file: "a.ts".into(), line: 1, message: String::new(), target: None, key: key.into() }
    }

    #[test]
    fn the_worst_finding_decides_the_outcome() {
        assert_eq!(Verdict::default().outcome(), Outcome::Pass);
        assert_eq!(Verdict::default().with([finding(Rule::R5, Severity::Note, "n")]).outcome(), Outcome::Pass);
        assert_eq!(Verdict::default().with([finding(Rule::R3, Severity::Ask, "a"), finding(Rule::R5, Severity::Note, "n")]).outcome(), Outcome::Ask);
        assert_eq!(Verdict::default().with([finding(Rule::R3, Severity::Ask, "a"), finding(Rule::R1, Severity::Block, "b")]).outcome(), Outcome::Deny);
    }

    #[test]
    fn an_accepted_exception_silences_its_finding_but_never_integrity() {
        let exceptions = Exceptions { keys: ["R1:x".to_string(), "R7:y".to_string()].into() };
        let verdict = Verdict::default().with([finding(Rule::R1, Severity::Block, "R1:x"), finding(Rule::R7, Severity::Block, "R7:y"), finding(Rule::R2, Severity::Block, "R2:z")]);
        let kept: Vec<Rule> = exceptions.filter(verdict).findings.iter().map(|finding| finding.rule).collect();
        assert_eq!(kept, [Rule::R7, Rule::R2]);
    }
}
