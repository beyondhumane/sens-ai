use std::process::Command;
use std::time::Duration;

use sens_canon::review::{self, BRIEF};
use serde_json::Value;

use crate::process;

pub const PATIENCE: Duration = Duration::from_secs(180);

const ASKING: &[&str] = &[
    "-p",
    "--model",
    "haiku",
    "--output-format",
    "json",
    "--max-turns",
    "3",
    "--tools",
    "",
    "--strict-mcp-config",
    "--disable-slash-commands",
    "--no-session-persistence",
    "--safe-mode",
    "--settings",
    "{\"disableAllHooks\":true}",
    "--thinking",
    "disabled",
];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reviewed {
    pub answer: Value,
    pub cost: f64,
}

pub trait Reviewer: Send + Sync {
    fn review(&self, prompt: &str) -> Result<Reviewed, String>;
}

type Launch = Box<dyn Fn() -> Command + Send + Sync>;

pub struct Haiku {
    launch: Launch,
    patience: Duration,
}

impl Haiku {
    pub fn launching(launch: impl Fn() -> Command + Send + Sync + 'static) -> Haiku {
        Haiku { launch: Box::new(launch), patience: PATIENCE }
    }

    pub fn waiting(self, patience: Duration) -> Haiku {
        Haiku { patience, ..self }
    }
}

impl Default for Haiku {
    fn default() -> Self {
        Haiku::launching(process::claude)
    }
}

pub fn answered(reply: &str) -> Result<Reviewed, String> {
    let reply: Value = serde_json::from_str(reply).map_err(|error| format!("the reviewer's reply is not JSON: {error}"))?;
    if reply["is_error"] == true {
        return Err(format!("the reviewer failed: {}", reply["errors"].as_array().and_then(|errors| errors.first()).and_then(Value::as_str).unwrap_or("no reason given")));
    }
    match &reply["structured_output"] {
        Value::Object(_) => Ok(Reviewed { answer: reply["structured_output"].clone(), cost: reply["total_cost_usd"].as_f64().unwrap_or(0.0) }),
        _ => Err("the reviewer gave no findings".into()),
    }
}

impl Reviewer for Haiku {
    fn review(&self, prompt: &str) -> Result<Reviewed, String> {
        let mut command = (self.launch)();
        command.args(ASKING).arg("--system-prompt").arg(BRIEF).arg("--json-schema").arg(review::schema());
        answered(&process::run_within(command, prompt, self.patience)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_structured_answer_counts_as_a_review() {
        let reviewed = answered(r#"{"is_error":false,"total_cost_usd":0.004,"structured_output":{"findings":[]}}"#).unwrap();
        assert_eq!((reviewed.answer["findings"].clone(), reviewed.cost), (serde_json::json!([]), 0.004));
        assert!(answered(r#"{"is_error":true,"errors":["Reached maximum number of turns (3)"]}"#).unwrap_err().contains("maximum number of turns"));
        assert!(answered(r#"{"is_error":false,"result":"looks fine"}"#).is_err());
        assert!(answered("not json").is_err());
    }

    #[test]
    fn a_reviewer_that_does_not_answer_in_time_is_stopped() {
        let silent = Haiku::launching(|| {
            let mut command = Command::new("node");
            command.args(["-e", "setTimeout(() => {}, 60000)", "--"]);
            command
        })
        .waiting(Duration::from_secs(2));
        let started = std::time::Instant::now();
        assert!(silent.review("hola").is_err());
        assert!(started.elapsed() < Duration::from_secs(20));
    }
}
