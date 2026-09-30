use std::path::Path;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sens_agent::chat::{Decision, Engine, Event, Message, Settings, Sink};
use sens_agent::session;
use sens_canon::verdict::Finding;
use serde_json::Value;

const NOBODY: &str = "No one is available to answer. Decide on your own and carry on.";
const TICK: Duration = Duration::from_millis(250);
const LINGER: Duration = Duration::from_secs(60);

#[derive(Debug, Default)]
pub struct Turn {
    pub finished: bool,
    pub ok: bool,
    pub error: String,
    pub millis: u64,
    pub turns: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub asked: u64,
    pub held: bool,
    pub circuit: Vec<String>,
    pub lingered: bool,
}

pub fn drive(engine: &Engine, root: &Path, prompt: &str, settings: Settings, patience: Duration, allowed: &[String]) -> Result<Turn, String> {
    let id = session::open(root)?;
    let (tell, heard) = mpsc::channel::<Event>();
    let tell = Mutex::new(tell);
    let sink: Sink = Arc::new(move |_, event| {
        if let Ok(tell) = tell.lock() {
            let _ = tell.send(event.clone());
        }
    });
    engine.send(root, &id, &Message { text: prompt.into(), ..Message::default() }, settings, sink)?;

    let until = Instant::now() + patience;
    let mut turn = Turn::default();
    let mut ended: Option<Instant> = None;
    loop {
        if Instant::now() > until {
            let _ = engine.stop(&id);
            turn.error = format!("no terminó en {} s", patience.as_secs());
            break;
        }
        match heard.recv_timeout(TICK) {
            Ok(event) => settle(engine, &id, &mut turn, event, allowed)?,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        if turn.finished && !engine.busy(&id) {
            if engine.tasks(&id).is_empty() {
                break;
            }
            if ended.get_or_insert_with(Instant::now).elapsed() > LINGER {
                let _ = engine.stop(&id);
                turn.lingered = true;
                break;
            }
        }
    }
    engine.forget(&id);
    Ok(turn)
}

fn permitted(tool: &str, input: &Value, allowed: &[String]) -> bool {
    tool == "sens.dependency" && input["key"].as_str().and_then(|key| key.strip_prefix("R3:")).is_some_and(|name| allowed.iter().any(|allowed| allowed == name))
}

fn heard(stage: &str, findings: &[Finding], suggested: usize) -> Vec<String> {
    if !findings.is_empty() {
        return findings.iter().map(|finding| format!("{stage}:{:?}", finding.rule)).collect();
    }
    vec![if suggested > 0 { format!("{stage}·{suggested}") } else { stage.to_string() }]
}

fn settle(engine: &Engine, id: &str, turn: &mut Turn, event: Event, allowed: &[String]) -> Result<(), String> {
    match event {
        Event::Asking { request, tool, input, .. } => {
            turn.asked += 1;
            let decision = Decision { allow: permitted(&tool, &input, allowed), message: NOBODY.into(), ..Decision::default() };
            engine.answer(id, &request, &decision)?;
        }
        Event::Held { .. } => {
            turn.held = true;
            turn.circuit.push("held".into());
        }
        Event::Canon { stage, findings, suggestions } => turn.circuit.extend(heard(&stage, &findings, suggestions.len())),
        Event::Finished { ok, millis, turns, tokens_in, tokens_out, error, .. } => {
            turn.finished = true;
            turn.ok = ok;
            turn.millis += millis;
            turn.turns += turns;
            turn.tokens_in += tokens_in;
            turn.tokens_out += tokens_out;
            if !error.is_empty() {
                turn.error = error;
            }
        }
        Event::Failed { reason } => {
            turn.finished = true;
            turn.error = reason;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_a_dependency_the_task_allows_is_accepted_for_the_person() {
        let allowed = vec!["dayjs".to_string()];
        assert!(permitted("sens.dependency", &json!({ "key": "R3:dayjs" }), &allowed));
        assert!(!permitted("sens.dependency", &json!({ "key": "R3:moment" }), &allowed));
        assert!(!permitted("sens.tests", &json!({ "key": "R3:dayjs" }), &allowed));
        assert!(!permitted("Bash", &json!({}), &allowed));
    }

    #[test]
    fn the_circuit_is_written_down_stage_by_stage() {
        let finding: Finding = serde_json::from_value(json!({ "rule": "R1", "severity": "Block", "file": "a.ts", "line": 1, "message": "m", "key": "k" })).unwrap();
        assert_eq!(heard("write", &[finding.clone(), finding], 0), ["write:R1", "write:R1"]);
        assert_eq!(heard("anticipated", &[], 3), ["anticipated·3"]);
        assert_eq!(heard("passed", &[], 0), ["passed"]);
    }
}
