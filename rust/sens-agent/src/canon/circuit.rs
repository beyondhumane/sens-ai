use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use sens_canon::judge::{judge_command, judge_since, judge_turn};
use sens_canon::orphans;
use sens_canon::review::Review;
use sens_canon::verdict::{Change, Finding, Rule, Severity, Verdict};
use sens_index::index::Index;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::checkpoint::{Checkpoints, Restored, Tree};
use super::keeper::{Keeper, Project};
use super::log::{self, Decision, Entry};
use super::review::Reviewer;
use super::state::{self, State};
use crate::chat::Event;
use crate::language;
use crate::said;

pub const PROMPT: &str = "sens-prompt";
pub const WRITE: &str = "sens-write";
pub const SHELL: &str = "sens-shell";
pub const LANDED: &str = "sens-landed";
pub const CLOSE: &str = "sens-close";
pub const MAX_ROUNDS: u32 = 3;
pub const BARRED_TOOLS: &str = "EnterWorktree,ExitWorktree";
const SHELL_TOOLS: &str = "Bash|PowerShell|mcp__sens__run_in_terminal|mcp__sens__write_terminal";

const WAIT: u64 = 3600;
const INDEX_PATIENCE: Duration = Duration::from_secs(30);
const UNJUDGED: [&str; 12] = ["Read", "Grep", "Glob", "WebFetch", "WebSearch", "TodoWrite", "ToolSearch", "Skill", "AskUserQuestion", "Write", "Edit", "Agent"];
const LOCKFILES: [&str; 7] = ["package-lock.json", "yarn.lock", "pnpm-lock.yaml", "Cargo.lock", "poetry.lock", "uv.lock", "composer.lock"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Suggested {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub signature: String,
    pub uses: usize,
}

pub trait Voice: Send + Sync {
    fn say(&self, event: Event);
    fn ask(&self, finding: &Finding) -> bool;
}

pub fn hooks() -> Value {
    let entry = |matcher: Option<&str>, id: &str| match matcher {
        Some(matcher) => json!({ "matcher": matcher, "hookCallbackIds": [id], "timeout": WAIT }),
        None => json!({ "hookCallbackIds": [id], "timeout": WAIT }),
    };
    json!({
        "UserPromptSubmit": [entry(None, PROMPT)],
        "PreToolUse": [entry(Some("Write|Edit|NotebookEdit"), WRITE), entry(Some(SHELL_TOOLS), SHELL)],
        "PostToolUse": [entry(None, LANDED)],
        "Stop": [entry(None, CLOSE)],
        "SubagentStop": [entry(None, CLOSE)]
    })
}

fn said(event: &str, text: String) -> Value {
    json!({ "hookSpecificOutput": { "hookEventName": event, "additionalContext": text } })
}

fn denied(text: String) -> Value {
    json!({ "hookSpecificOutput": { "hookEventName": "PreToolUse", "permissionDecision": "deny", "permissionDecisionReason": text } })
}

fn blocked(text: String) -> Value {
    json!({ "decision": "block", "reason": text })
}

fn listed(findings: &[&Finding]) -> String {
    findings
        .iter()
        .enumerate()
        .map(|(at, finding)| match &finding.target {
            Some(target) => format!("{}. {}\n   Existing code at {}:{}:\n{}", at + 1, finding.message, target.file, target.line, target.excerpt.lines().map(|line| format!("   | {line}")).collect::<Vec<_>>().join("\n")),
            None => format!("{}. {}", at + 1, finding.message),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn commits(command: &str) -> bool {
    command.split(['\n', ';', '&', '|']).any(|part| {
        let mut words = part.split_whitespace();
        while let Some(word) = words.next() {
            if word != "git" {
                continue;
            }
            while let Some(next) = words.next() {
                match next {
                    "-c" | "-C" => {
                        words.next();
                    }
                    flag if flag.starts_with('-') => {}
                    verb => return verb == "commit" || verb == "push",
                }
            }
        }
        false
    })
}

#[derive(Default)]
struct Judged {
    blocks: Vec<Finding>,
    notes: Vec<Finding>,
    restored: Vec<String>,
}

pub struct Circuit {
    work: PathBuf,
    session: String,
    keeper: Arc<Keeper>,
    checkpoints: Option<Checkpoints>,
    reviewer: Option<Arc<dyn Reviewer>>,
}

impl Circuit {
    pub fn new(work: &Path, session: &str, keeper: Arc<Keeper>, resumed: bool) -> Circuit {
        let circuit = Circuit { work: work.to_path_buf(), session: session.to_string(), keeper, checkpoints: Checkpoints::open(work), reviewer: None };
        if !resumed {
            circuit.keeper.exclusive(&circuit.work, || {
                let mut state = State::load(&circuit.work);
                state.canon.insert(circuit.session.clone(), sens_canon::VERSION.to_string());
                let _ = state.save(&circuit.work);
            });
        }
        circuit
    }

    pub fn reviewed_by(self, reviewer: Arc<dyn Reviewer>) -> Circuit {
        Circuit { reviewer: Some(reviewer), ..self }
    }

    pub fn retry(&self, voice: &dyn Voice) -> Value {
        self.answer(voice, CLOSE, &json!({ "hook_event_name": "Stop", "stop_hook_active": false, "background_tasks": [] }))
    }

    pub fn answer(&self, voice: &dyn Voice, callback: &str, input: &Value) -> Value {
        self.keeper.exclusive(&self.work, || match callback {
            PROMPT => self.prompt(voice, input),
            WRITE => self.write(voice, input),
            SHELL => self.shell(voice, input),
            LANDED => self.landed(voice, input),
            CLOSE => self.close(voice, input),
            _ => json!({}),
        })
    }

    fn relative(&self, written: &str) -> Option<String> {
        let path = Path::new(written);
        let absolute = if path.is_absolute() { path.to_path_buf() } else { self.work.join(path) };
        let flat = |path: &Path| path.to_string_lossy().replace('\\', "/");
        let (full, base) = (flat(&absolute), flat(&self.work));
        let base = base.trim_end_matches('/');
        let inside = if cfg!(windows) { full.to_lowercase().starts_with(&format!("{}/", base.to_lowercase())) } else { full.starts_with(&format!("{base}/")) };
        inside.then(|| full[base.len() + 1..].to_string())
    }

    fn project(&self, patience: Duration) -> Option<Arc<Project>> {
        self.keeper.ready(&self.work, patience)
    }

    fn snapshot(&self, state: &mut State, label: &str) -> Option<Tree> {
        state.step += 1;
        self.checkpoints.as_ref()?.snapshot(&format!("{}/{}-{}-{label}", self.session, state.turn, state.step)).ok()
    }

    fn logged(&self, stage: &str, entries: Vec<Entry>) {
        let state = State::load(&self.work);
        let canon = state.canon.get(&self.session).cloned().unwrap_or_else(|| sens_canon::VERSION.to_string());
        let stamped: Vec<Entry> = entries.into_iter().map(|entry| Entry { session: self.session.clone(), turn: state.turn, canon: canon.clone(), stage: stage.to_string(), round: state.rounds, ..entry }).collect();
        let _ = log::write(&self.work, &stamped);
    }

    fn weigh(&self, voice: &dyn Voice, stage: &str, verdict: Verdict, undo: impl Fn(&Finding) -> bool) -> Judged {
        let mut exceptions = state::exceptions(&self.work);
        let mut considered = state::considered(&self.work);
        let mut judged = Judged { blocks: Vec::new(), notes: Vec::new(), restored: Vec::new() };
        let mut entries = Vec::new();
        for finding in verdict.findings {
            let decision = match finding.severity {
                Severity::Block => Decision::Blocked,
                Severity::Consider if considered.contains(&finding.key) => Decision::Kept,
                Severity::Consider => Decision::Considered,
                Severity::Note => Decision::Noted,
                Severity::Ask => Decision::Refused,
            };
            let mut decided = log::entry(&finding, decision);
            match finding.severity {
                Severity::Block => {
                    if finding.rule == Rule::R7 && undo(&finding) {
                        judged.restored.push(finding.file.clone());
                    }
                    judged.blocks.push(finding);
                }
                Severity::Ask if voice.ask(&finding) => {
                    decided.decision = Some(Decision::Allowed);
                    exceptions.keys.insert(finding.key.clone());
                }
                Severity::Ask => {
                    if undo(&finding) {
                        judged.restored.push(finding.file.clone());
                    }
                    judged.blocks.push(Finding { message: format!("The person said no. {}", finding.message), severity: Severity::Block, ..finding });
                }
                Severity::Consider if considered.contains(&finding.key) => {
                    judged.notes.push(Finding { message: format!("Kept after Sens showed it. {}", finding.message), severity: Severity::Note, ..finding });
                }
                Severity::Consider => {
                    considered.insert(finding.key.clone());
                    judged.blocks.push(finding);
                }
                Severity::Note => judged.notes.push(finding),
            }
            entries.push(decided);
        }
        self.logged(stage, entries);
        let _ = state::save_considered(&self.work, &considered);
        let _ = state::save_exceptions(&self.work, &exceptions);
        judged
    }

    fn reply(&self, voice: &dyn Voice, stage: &str, judged: &Judged, event: &str) -> Value {
        voice.say(Event::Canon { stage: stage.to_string(), findings: judged.blocks.iter().chain(&judged.notes).cloned().collect(), suggestions: Vec::new() });
        if !judged.blocks.is_empty() {
            let mut text = format!("Sens stopped this:\n{}", listed(&judged.blocks.iter().collect::<Vec<_>>()));
            if !judged.restored.is_empty() {
                text.push_str(&format!("\nSens put back: {}.", judged.restored.join(", ")));
            }
            return match event {
                "PreToolUse" => denied(text),
                _ => blocked(text),
            };
        }
        if judged.notes.is_empty() {
            return json!({});
        }
        said(event, format!("Sens noticed, without blocking:\n{}", listed(&judged.notes.iter().collect::<Vec<_>>())))
    }

    fn prompt(&self, voice: &dyn Voice, input: &Value) -> Value {
        let mut state = State::load(&self.work);
        state.turn += 1;
        state.request = input["prompt"].as_str().unwrap_or_default().to_string();
        let project = self.project(Duration::ZERO);
        let now = self.snapshot(&mut state, "start");
        if state.approved.is_none() {
            state.approved = now.clone();
            if let Some(project) = &project {
                state.dead = orphans::dead(&project.index).into_iter().collect();
                state.dead_known = true;
            }
        }
        state.seen = now;
        let mut context = Vec::new();
        if state.canon.get(&self.session).map(String::as_str) != Some(sens_canon::VERSION) {
            context.push(sens_canon::CANON.to_string());
            state.canon.insert(self.session.clone(), sens_canon::VERSION.to_string());
        }
        if let Some(project) = &project {
            let found: Vec<Suggested> = project
                .catalog
                .relevant(&state.request)
                .iter()
                .map(|suggestion| {
                    let symbol = &project.index.symbols[suggestion.symbol];
                    Suggested { name: symbol.name.clone(), file: symbol.file.clone(), line: symbol.line, signature: symbol.signature.clone(), uses: suggestion.uses }
                })
                .collect();
            if !found.is_empty() {
                let lines: Vec<String> = found.iter().map(|found| format!("- `{}` — {}:{} · used {} times", found.signature, found.file, found.line, found.uses)).collect();
                context.push(format!("Already in this project and related to the request (reuse before writing new code):\n{}", lines.join("\n")));
                voice.say(Event::Canon { stage: "anticipated".into(), findings: Vec::new(), suggestions: found });
            }
        }
        let _ = state.save(&self.work);
        if context.is_empty() { json!({}) } else { said("UserPromptSubmit", context.join("\n\n")) }
    }

    fn write(&self, voice: &dyn Voice, input: &Value) -> Value {
        let tool = &input["tool_input"];
        let Some(path) = tool["file_path"].as_str().and_then(|written| self.relative(written)) else {
            return json!({});
        };
        let before = std::fs::read_to_string(self.work.join(&path)).ok();
        let after = match input["tool_name"].as_str() {
            Some("Write") => tool["content"].as_str().map(str::to_string),
            Some("Edit") => edited(before.as_deref(), tool),
            _ => None,
        };
        let Some(after) = after else {
            return json!({});
        };
        let empty = Index::default();
        let project = self.project(Duration::ZERO);
        let index = project.as_ref().map_or(&empty, |project| &project.index);
        let approved = self.as_approved(State::load(&self.work).approved.as_ref(), &path, before.as_deref());
        let change = Change { path, before, after: Some(after) };
        let verdict = judge_since(index, &change, approved.as_deref(), &state::rules(&self.work), &state::exceptions(&self.work));
        let judged = self.weigh(voice, "write", verdict, |_| false);
        self.reply(voice, "write", &judged, "PreToolUse")
    }

    fn shell(&self, voice: &dyn Voice, input: &Value) -> Value {
        let typed = &input["tool_input"];
        let command = typed["command"].as_str().or(typed["text"].as_str()).unwrap_or_default();
        let judged = self.weigh(voice, "shell", judge_command(command), |_| false);
        if !judged.blocks.is_empty() {
            return self.reply(voice, "shell", &judged, "PreToolUse");
        }
        if !commits(command) {
            return json!({});
        }
        match self.audit(voice, false, None) {
            Audited::Untouched | Audited::Clean | Audited::Pending => json!({}),
            Audited::Blocked(judged) | Audited::Held(judged) => {
                voice.say(Event::Canon { stage: "commit".into(), findings: judged.blocks.clone(), suggestions: Vec::new() });
                denied(format!("Sens has not approved these changes, so they cannot be committed yet:\n{}", listed(&judged.blocks.iter().collect::<Vec<_>>())))
            }
            Audited::Unjudged(reason) => denied(reason),
        }
    }

    fn landed(&self, voice: &dyn Voice, input: &Value) -> Value {
        if UNJUDGED.contains(&input["tool_name"].as_str().unwrap_or_default()) {
            return json!({});
        }
        let Some(checkpoints) = &self.checkpoints else {
            return json!({});
        };
        let mut state = State::load(&self.work);
        let Some(seen) = state.seen.clone() else {
            return json!({});
        };
        let Some(now) = self.snapshot(&mut state, "landed") else {
            return json!({});
        };
        let changes = checkpoints.changes(&seen, &now).unwrap_or_default();
        state.seen = Some(now);
        let _ = state.save(&self.work);
        if changes.is_empty() {
            return json!({});
        }
        let empty = Index::default();
        let project = self.project(Duration::ZERO);
        let index = project.as_ref().map_or(&empty, |project| &project.index);
        let (rules, exceptions) = (state::rules(&self.work), state::exceptions(&self.work));
        let approved = |change: &Change| self.as_approved(state.approved.as_ref(), &change.path, change.before.as_deref());
        let verdict = changes.iter().fold(Verdict::default(), |verdict, change| verdict.with(judge_since(index, change, approved(change).as_deref(), &rules, &exceptions).findings));
        let changed: HashSet<&str> = changes.iter().map(|change| change.path.as_str()).collect();
        let judged = self.weigh(voice, "landed", verdict, |finding| self.put_back(&seen, &finding.file, &changed));
        if !judged.restored.is_empty() {
            let mut state = State::load(&self.work);
            state.seen = self.snapshot(&mut state, "restored");
            let _ = state.save(&self.work);
        }
        self.reply(voice, "landed", &judged, "PostToolUse")
    }

    fn as_approved(&self, approved: Option<&Tree>, path: &str, current: Option<&str>) -> Option<String> {
        match (&self.checkpoints, approved) {
            (Some(checkpoints), Some(tree)) => checkpoints.read(tree, path).map(|bytes| String::from_utf8_lossy(&bytes).into_owned()),
            _ => current.map(str::to_string),
        }
    }

    fn put_back(&self, tree: &Tree, path: &str, changed: &HashSet<&str>) -> bool {
        let Some(checkpoints) = &self.checkpoints else {
            return false;
        };
        let folder = path.rsplit_once('/').map_or("", |(folder, _)| folder);
        let locks = LOCKFILES.iter().map(|lock| if folder.is_empty() { lock.to_string() } else { format!("{folder}/{lock}") }).filter(|lock| changed.contains(lock.as_str()));
        std::iter::once(path.to_string()).chain(locks).all(|file| checkpoints.put_back(tree, &file).is_ok())
    }

    fn close(&self, voice: &dyn Voice, input: &Value) -> Value {
        let helper = (input["hook_event_name"] == "SubagentStop").then(|| input["agent_id"].as_str().unwrap_or("helper").to_string());
        let running = input["background_tasks"].as_array().is_some_and(|tasks| !tasks.is_empty());
        match self.audit(voice, helper.is_some() || running, helper.as_deref()) {
            Audited::Untouched => json!({}),
            Audited::Clean => {
                voice.say(Event::Canon { stage: "passed".into(), findings: Vec::new(), suggestions: Vec::new() });
                json!({})
            }
            Audited::Pending => {
                voice.say(Event::Canon { stage: "pending".into(), findings: Vec::new(), suggestions: Vec::new() });
                json!({})
            }
            Audited::Blocked(judged) => self.reply(voice, "blocked", &judged, "Stop"),
            Audited::Held(judged) => {
                voice.say(Event::Held { findings: judged.blocks });
                json!({})
            }
            Audited::Unjudged(reason) => {
                let finding = unjudged(reason);
                self.logged("close", vec![log::entry(&finding, Decision::Unjudged)]);
                voice.say(Event::Held { findings: vec![finding] });
                json!({})
            }
        }
    }

    fn audit(&self, voice: &dyn Voice, provisional: bool, helper: Option<&str>) -> Audited {
        let Some(checkpoints) = &self.checkpoints else {
            return Audited::Clean;
        };
        let mut state = State::load(&self.work);
        let Some(now) = self.snapshot(&mut state, "close") else {
            return Audited::Unjudged(said!(
                en: "Sens could not take a snapshot of the project, so it could not judge these changes.",
                es: "Sens no pudo hacer una foto del proyecto, así que no pudo juzgar estos cambios.",
                fr: "Sens n’a pas pu prendre d’instantané du projet et n’a donc pas pu juger ces modifications.",
                de: "Sens konnte keinen Schnappschuss des Projekts machen und diese Änderungen daher nicht beurteilen.",
                ja: "Sens はプロジェクトのスナップショットを取れなかったため、これらの変更を判断できませんでした。",
                zh: "Sens 无法为项目拍摄快照，因此无法评判这些更改。",
            ));
        };
        let approved = state.approved.clone().unwrap_or_else(|| now.clone());
        let changes = checkpoints.changes(&approved, &now).unwrap_or_default();
        state.seen = Some(now.clone());
        state.end = Some(now.clone());
        if changes.is_empty() {
            if !provisional {
                state.rounds = 0;
                state.held = None;
            }
            let _ = state.save(&self.work);
            return Audited::Untouched;
        }
        if self.project(INDEX_PATIENCE).is_none() {
            let _ = state.save(&self.work);
            return Audited::Unjudged(said!(
                en: "Sens could not finish indexing the project in time, so it could not judge these changes.",
                es: "Sens no terminó de indexar el proyecto a tiempo, así que no pudo juzgar estos cambios.",
                fr: "Sens n’a pas fini d’indexer le projet à temps et n’a donc pas pu juger ces modifications.",
                de: "Sens konnte das Projekt nicht rechtzeitig indizieren und diese Änderungen daher nicht beurteilen.",
                ja: "Sens はプロジェクトのインデックス作成が間に合わず、これらの変更を判断できませんでした。",
                zh: "Sens 未能及时完成项目索引，因此无法评判这些更改。",
            ));
        }
        let project = self.keeper.refresh(&self.work);
        let dead: orphans::Dead = if state.dead_known { state.dead.iter().cloned().collect() } else { orphans::dead(&project.index) };
        let (verdict, _) = judge_turn(&project.index, &changes, &dead, &state::rules(&self.work), &state::exceptions(&self.work));
        let changed: HashSet<&str> = changes.iter().map(|change| change.path.as_str()).collect();
        let mut judged = self.weigh(voice, "close", verdict, |finding| self.put_back(&approved, &finding.file, &changed));
        if judged.blocks.is_empty() && !provisional {
            match self.reviewed(voice, &project, &state.request, &approved, &now) {
                Ok(reviewed) => {
                    judged.blocks = reviewed.blocks;
                    judged.notes.extend(reviewed.notes);
                }
                Err(reason) => {
                    let _ = state.save(&self.work);
                    return Audited::Unjudged(said!(
                        en: "Sens’s reviewer could not judge these changes: {reason}",
                        es: "El revisor de Sens no pudo juzgar estos cambios: {reason}",
                        fr: "Le relecteur de Sens n’a pas pu juger ces modifications : {reason}",
                        de: "Die Prüfung von Sens konnte diese Änderungen nicht beurteilen: {reason}",
                        ja: "Sens のレビューはこれらの変更を判断できませんでした: {reason}",
                        zh: "Sens 的审查无法评判这些更改：{reason}",
                    ));
                }
            }
        }
        if judged.blocks.is_empty() {
            if provisional {
                let _ = state.save(&self.work);
                return Audited::Pending;
            }
            state.approved = Some(now);
            state.dead = orphans::dead(&project.index).into_iter().collect();
            state.dead_known = true;
            state.rounds = 0;
            state.held = None;
            let _ = state.save(&self.work);
            return Audited::Clean;
        }
        let rounds = match helper {
            Some(helper) => state.helper_rounds.entry(helper.to_string()).or_default(),
            None => &mut state.rounds,
        };
        *rounds += 1;
        if *rounds <= MAX_ROUNDS {
            let _ = state.save(&self.work);
            return Audited::Blocked(judged);
        }
        *rounds = 0;
        if helper.is_some() {
            let _ = state.save(&self.work);
            return Audited::Pending;
        }
        state.held = Some(judged.blocks.clone());
        let _ = state.save(&self.work);
        self.logged("close", judged.blocks.iter().map(|finding| log::entry(finding, Decision::Held)).collect());
        Audited::Held(judged)
    }
}

impl Circuit {
    fn reviewed(&self, voice: &dyn Voice, project: &Project, request: &str, approved: &Tree, now: &Tree) -> Result<Judged, String> {
        let (Some(reviewer), Some(checkpoints)) = (&self.reviewer, &self.checkpoints) else {
            return Ok(Judged::default());
        };
        let diff = checkpoints.diff(approved, now)?;
        let review = Review::of(&project.index, &project.catalog, request, &diff);
        let reviewed = reviewer.review(&review.prompt(language::now().tongue()))?;
        self.logged("reviewed", vec![Entry { at: log::now(), decision: Some(Decision::Reviewed), cost: reviewed.cost, ..Entry::default() }]);
        let findings = review.findings(&reviewed.answer);
        voice.say(Event::Canon { stage: "reviewed".into(), findings: findings.clone(), suggestions: Vec::new() });
        let verdict = state::exceptions(&self.work).filter(Verdict { findings });
        Ok(self.weigh(voice, "reviewed", verdict, |_| false))
    }
}

enum Audited {
    Untouched,
    Clean,
    Pending,
    Blocked(Judged),
    Held(Judged),
    Unjudged(String),
}

fn unjudged(reason: String) -> Finding {
    Finding { rule: Rule::R7, severity: Severity::Block, file: String::new(), line: 0, message: reason, target: None, key: "unjudged".into() }
}

fn edited(before: Option<&str>, tool: &Value) -> Option<String> {
    let before = before?;
    let old = tool["old_string"].as_str()?;
    let new = tool["new_string"].as_str()?;
    if old.is_empty() || !before.contains(old) {
        return None;
    }
    Some(match tool["replace_all"].as_bool() {
        Some(true) => before.replace(old, new),
        _ => before.replacen(old, new, 1),
    })
}

pub fn held(work: &Path) -> Option<Vec<Finding>> {
    State::load(work).held
}

pub fn fix_request(work: &Path) -> Option<String> {
    let mut state = State::load(work);
    let held = state.held.take()?;
    state.rounds = 0;
    state.save(work).ok()?;
    Some(format!("Sens held your last turn. Fix what it found, then finish again:\n{}", listed(&held.iter().collect::<Vec<_>>())))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Excepted {
    pub key: String,
    pub rule: Option<Rule>,
    pub file: String,
    pub since: u64,
}

pub fn exceptions(work: &Path) -> Vec<Excepted> {
    let logged = log::read(work);
    state::exceptions(work)
        .keys
        .into_iter()
        .map(|key| {
            let granted = logged.iter().rev().find(|entry| entry.key == key && matches!(entry.decision, Some(Decision::Accepted | Decision::Allowed)));
            Excepted { rule: granted.and_then(|entry| entry.rule), file: granted.map(|entry| entry.file.clone()).unwrap_or_default(), since: granted.map_or(0, |entry| entry.at), key }
        })
        .collect()
}

pub fn retract(work: &Path, key: &str) -> Result<(), String> {
    let mut exceptions = state::exceptions(work);
    exceptions.keys.remove(key);
    state::save_exceptions(work, &exceptions)
}

pub fn accept(work: &Path) -> Result<(), String> {
    let mut state = State::load(work);
    let Some(held) = state.held.take() else {
        return Ok(());
    };
    let _ = log::write(work, &held.iter().map(|finding| Entry { turn: state.turn, ..log::entry(finding, Decision::Accepted) }).collect::<Vec<_>>());
    let mut exceptions = state::exceptions(work);
    exceptions.keys.extend(held.into_iter().filter(|finding| finding.rule != Rule::R7).map(|finding| finding.key));
    state::save_exceptions(work, &exceptions)?;
    state.approved = state.end.clone().or(state.approved);
    state.rounds = 0;
    state.save(work)
}

pub fn undo(work: &Path) -> Result<Restored, String> {
    let mut state = State::load(work);
    let (Some(approved), Some(end)) = (state.approved.clone(), state.end.clone()) else {
        return Ok(Restored::default());
    };
    let checkpoints = Checkpoints::open(work).ok_or_else(|| {
        said!(
            en: "git is not available, so Sens cannot undo these changes",
            es: "git no está disponible, así que Sens no puede deshacer estos cambios",
            fr: "git n’est pas disponible, Sens ne peut donc pas annuler ces modifications",
            de: "git ist nicht verfügbar, daher kann Sens diese Änderungen nicht rückgängig machen",
            ja: "git が使えないため、Sens はこれらの変更を元に戻せません",
            zh: "git 不可用，因此 Sens 无法撤销这些更改",
        )
    })?;
    let restored = checkpoints.restore(&approved, &end)?;
    let undone: Vec<Entry> = state.held.iter().flatten().map(|finding| Entry { turn: state.turn, ..log::entry(finding, Decision::Undone) }).collect();
    let _ = log::write(work, &undone);
    state.held = None;
    state.rounds = 0;
    state.seen = Some(approved);
    state.save(work)?;
    Ok(restored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    const PLAIN: &str = r#"export const plain = (text) => text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();"#;

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

    const REUSES: &str = "import { totals } from './lib/totals.ts';\nexport const report = () => totals([], 2);\n";

    #[derive(Default)]
    struct Ear {
        said: Mutex<Vec<Event>>,
        answers: Mutex<VecDeque<bool>>,
        asked: Mutex<Vec<Finding>>,
    }

    impl Voice for Ear {
        fn say(&self, event: Event) {
            self.said.lock().unwrap().push(event);
        }

        fn ask(&self, finding: &Finding) -> bool {
            self.asked.lock().unwrap().push(finding.clone());
            self.answers.lock().unwrap().pop_front().unwrap_or(false)
        }
    }

    impl Ear {
        fn answering(answers: &[bool]) -> Ear {
            Ear { answers: Mutex::new(answers.iter().copied().collect()), ..Ear::default() }
        }

        fn stages(&self) -> Vec<String> {
            self.said
                .lock()
                .unwrap()
                .iter()
                .map(|event| match event {
                    Event::Canon { stage, .. } => stage.clone(),
                    Event::Held { .. } => "held".into(),
                    _ => "other".into(),
                })
                .collect()
        }
    }

    fn project(name: &str) -> (PathBuf, Arc<Keeper>) {
        let root = std::env::temp_dir().join("sens-circuit").join(name);
        let _ = std::fs::remove_dir_all(&root);
        let files = [
            ("package.json", r#"{ "main": "src/index.ts", "dependencies": { "dayjs": "1" } }"#),
            ("src/index.ts", "import { totals } from './lib/totals.ts';\nimport { plain } from './lib/text.ts';\ntotals([], 1);\nplain('a');\n"),
            ("src/lib/totals.ts", TOTALS),
            ("src/lib/text.ts", PLAIN),
        ];
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        let keeper = Arc::new(Keeper::default());
        keeper.ready(&root, Duration::from_secs(30)).unwrap();
        (root, keeper)
    }

    fn started(name: &str) -> (PathBuf, Circuit, Ear) {
        let (root, keeper) = project(name);
        let circuit = Circuit::new(&root, "s1", keeper, false);
        let ear = Ear::default();
        circuit.answer(&ear, PROMPT, &json!({ "prompt": "hola" }));
        (root, circuit, ear)
    }

    fn write(path: &Path, content: &str) -> Value {
        json!({ "hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": { "file_path": path.to_string_lossy(), "content": content } })
    }

    fn shell(command: &str) -> Value {
        json!({ "hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": { "command": command } })
    }

    fn decision(output: &Value) -> &str {
        output["hookSpecificOutput"]["permissionDecision"].as_str().or(output["decision"].as_str()).unwrap_or("allow")
    }

    fn close() -> Value {
        json!({ "hook_event_name": "Stop", "stop_hook_active": false, "background_tasks": [] })
    }

    #[test]
    fn a_request_brings_what_already_exists_and_an_old_session_gets_the_canon_once() {
        let (root, keeper) = project("prompt");
        let fresh = Circuit::new(&root, "s1", keeper.clone(), false);
        let ear = Ear::default();
        let told = fresh.answer(&ear, PROMPT, &json!({ "prompt": "show the totals of the rows" }));
        let context = told["hookSpecificOutput"]["additionalContext"].as_str().unwrap();
        assert!(context.contains("totals(rows: Row[], limit: number)") && !context.contains("# Sens Canon"), "{context}");
        assert_eq!(ear.stages(), ["anticipated"]);
        assert!(State::load(&root).approved.is_some());

        let old = Circuit::new(&root, "s2", keeper, true);
        let first = old.answer(&ear, PROMPT, &json!({ "prompt": "hola" }));
        assert!(first["hookSpecificOutput"]["additionalContext"].as_str().unwrap().contains("# Sens Canon"));
        assert_eq!(old.answer(&ear, PROMPT, &json!({ "prompt": "hola" })), json!({}));
    }

    #[test]
    fn a_copied_function_is_denied_with_the_code_to_reuse_and_outside_paths_pass() {
        let (root, circuit, ear) = started("write");
        let copy = TOTALS.replace("totals", "summarize");
        let output = circuit.answer(&ear, WRITE, &write(&root.join("src/report.ts"), &copy));
        assert_eq!(decision(&output), "deny");
        let reason = output["hookSpecificOutput"]["permissionDecisionReason"].as_str().unwrap();
        assert!(reason.contains("Call `totals`") && reason.contains("| export function totals("), "{reason}");
        assert!(ear.stages().contains(&"write".to_string()));
        let logged = log::read(&root);
        assert!(logged.iter().any(|entry| entry.stage == "write" && entry.rule == Some(Rule::R1) && entry.decision == Some(Decision::Blocked) && entry.session == "s1"), "{logged:?}");
        let elsewhere = std::env::temp_dir().join("sens-circuit-outside.ts");
        assert_eq!(circuit.answer(&ear, WRITE, &write(&elsewhere, &copy)), json!({}));
    }

    #[test]
    fn a_small_helper_written_again_is_shown_once_and_passes_if_the_model_insists() {
        let (root, circuit, ear) = started("consider");
        let again = r#"export const lower = (text: string) => text.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase();"#;
        let first = circuit.answer(&ear, WRITE, &write(&root.join("src/bar.ts"), again));
        assert_eq!(decision(&first), "deny");
        let reason = first["hookSpecificOutput"]["permissionDecisionReason"].as_str().unwrap();
        assert!(reason.contains("use `plain`") && reason.contains("src/lib/text.ts"), "{reason}");
        let second = circuit.answer(&ear, WRITE, &write(&root.join("src/bar.ts"), again));
        assert_eq!(decision(&second), "allow", "{second}");
        let decided: Vec<Option<Decision>> = log::read(&root).into_iter().map(|entry| entry.decision).collect();
        assert_eq!(decided, [Some(Decision::Considered), Some(Decision::Kept)]);
        assert_eq!(log::avoided(&root, 0).copies, 0);
    }

    #[test]
    fn a_new_dependency_waits_for_the_person_and_their_yes_is_remembered() {
        let (root, circuit, _) = started("dependency");
        let manifest = r#"{ "main": "src/index.ts", "dependencies": { "dayjs": "1", "moment": "2" } }"#;
        let no = Ear::answering(&[false]);
        let refused = circuit.answer(&no, WRITE, &write(&root.join("package.json"), manifest));
        assert_eq!(decision(&refused), "deny");
        assert!(refused["hookSpecificOutput"]["permissionDecisionReason"].as_str().unwrap().contains("The person said no"));
        assert_eq!(no.asked.lock().unwrap()[0].key, "R3:moment");
        let yes = Ear::answering(&[true]);
        assert_eq!(circuit.answer(&yes, WRITE, &write(&root.join("package.json"), manifest)), json!({}));
        assert!(state::exceptions(&root).keys.contains("R3:moment"));
    }

    #[test]
    fn a_test_written_in_this_turn_can_be_reshaped_but_one_sens_approved_is_asked_about() {
        let (root, keeper) = project("own-tests");
        let approved = "import { test } from 'node:test';
import assert from 'node:assert/strict';

test('plain', () => {
  assert.equal(plain('É'), 'e');
});
";
        let spec = root.join("test/text.test.ts");
        std::fs::create_dir_all(spec.parent().unwrap()).unwrap();
        std::fs::write(&spec, approved).unwrap();
        let circuit = Circuit::new(&root, "s1", keeper, false);
        let ear = Ear::default();
        circuit.answer(&ear, PROMPT, &json!({ "prompt": "hola" }));
        let grown = format!("{approved}
test('plain again', () => {{
  assert.equal(plain('Ñ'), 'n');
  assert.equal(plain('Ü'), 'u');
}});
");
        assert_eq!(decision(&circuit.answer(&ear, WRITE, &write(&spec, &grown))), "allow");
        std::fs::write(&spec, &grown).unwrap();
        let reshaped = grown.replace("  assert.equal(plain('Ü'), 'u');
", "");
        assert_eq!(decision(&circuit.answer(&ear, WRITE, &write(&spec, &reshaped))), "allow");
        assert_eq!(decision(&circuit.answer(&ear, WRITE, &write(&spec, "import { test } from 'node:test';
"))), "deny");
        assert_eq!(ear.asked.lock().unwrap().len(), 1);
    }

    #[test]
    fn protected_places_are_denied_to_writes_and_commands() {
        let (root, circuit, ear) = started("protected");
        assert_eq!(decision(&circuit.answer(&ear, WRITE, &write(&root.join(".claude/settings.local.json"), "{}"))), "deny");
        assert_eq!(decision(&circuit.answer(&ear, SHELL, &shell("rm -rf .sens"))), "deny");
        assert_eq!(decision(&circuit.answer(&ear, SHELL, &shell("git worktree add ../x"))), "deny");
        assert_eq!(circuit.answer(&ear, SHELL, &shell("npm test")), json!({}));
        let typed = json!({ "hook_event_name": "PreToolUse", "tool_name": "mcp__sens__write_terminal", "tool_input": { "terminal": 1, "text": "rm -rf .sens" } });
        assert_eq!(decision(&circuit.answer(&ear, SHELL, &typed)), "deny");
        let hooks = hooks();
        let watched = hooks["PreToolUse"][1]["matcher"].as_str().unwrap().split('|').collect::<Vec<_>>();
        assert!(["Bash", "PowerShell", "mcp__sens__run_in_terminal", "mcp__sens__write_terminal"].iter().all(|tool| watched.contains(tool)));
    }

    #[test]
    fn a_commit_waits_for_the_changes_to_pass() {
        let (root, circuit, ear) = started("commit");
        let committing = shell("git add -A && git -c user.name=x commit -m report");
        std::fs::write(root.join("src/report.ts"), TOTALS.replace("totals", "summarize")).unwrap();
        let output = circuit.answer(&ear, SHELL, &committing);
        assert_eq!(decision(&output), "deny");
        assert!(output["hookSpecificOutput"]["permissionDecisionReason"].as_str().unwrap().contains("cannot be committed"));
        std::fs::write(root.join("src/report.ts"), REUSES).unwrap();
        assert_eq!(circuit.answer(&ear, SHELL, &committing), json!({}));
        let state = State::load(&root);
        assert_eq!(state.approved, state.end);
    }

    #[test]
    fn what_a_command_writes_is_judged_and_a_touched_setting_is_put_back() {
        let (root, circuit, ear) = started("landed");
        std::fs::write(root.join("src/report.ts"), TOTALS.replace("totals", "summarize")).unwrap();
        std::fs::create_dir_all(root.join(".claude")).unwrap();
        std::fs::write(root.join(".claude/settings.local.json"), r#"{ "disableAllHooks": true }"#).unwrap();
        let output = circuit.answer(&ear, LANDED, &json!({ "hook_event_name": "PostToolUse", "tool_name": "Bash", "tool_input": { "command": "node make.js" } }));
        assert_eq!(decision(&output), "block");
        let reason = output["reason"].as_str().unwrap();
        assert!(reason.contains("Call `totals`") && reason.contains("Sens put back: .claude/settings.local.json"), "{reason}");
        assert!(!root.join(".claude/settings.local.json").exists());
        assert_eq!(circuit.answer(&ear, LANDED, &json!({ "tool_name": "Read", "tool_input": {} })), json!({}));
    }

    #[test]
    fn three_blocked_endings_then_the_turn_is_held_and_the_person_decides() {
        let (root, circuit, ear) = started("held");
        std::fs::write(root.join("src/report.ts"), TOTALS.replace("totals", "summarize")).unwrap();
        for _ in 0..MAX_ROUNDS {
            assert_eq!(decision(&circuit.answer(&ear, CLOSE, &close())), "block");
        }
        assert_eq!(circuit.answer(&ear, CLOSE, &close()), json!({}));
        assert!(ear.stages().ends_with(&["held".to_string()]));
        assert_eq!(held(&root).unwrap()[0].rule, Rule::R1);

        let restored = undo(&root).unwrap();
        assert_eq!(restored.restored, ["src/report.ts"]);
        assert!(!root.join("src/report.ts").exists() && held(&root).is_none());
    }

    #[test]
    fn asking_for_a_fix_hands_the_findings_back_with_fresh_rounds_and_retrying_judges_again() {
        let (root, circuit, ear) = started("fix");
        std::fs::write(root.join("src/report.ts"), TOTALS.replace("totals", "summarize")).unwrap();
        for _ in 0..=MAX_ROUNDS {
            circuit.answer(&ear, CLOSE, &close());
        }
        assert!(held(&root).is_some());
        let asked = fix_request(&root).unwrap();
        assert!(asked.contains("Call `totals`"), "{asked}");
        let state = State::load(&root);
        assert!(state.held.is_none() && state.rounds == 0);
        assert!(fix_request(&root).is_none());
        std::fs::write(root.join("src/report.ts"), REUSES).unwrap();
        assert_eq!(decision(&circuit.retry(&ear)), "allow");
        assert_eq!(ear.stages().last().map(String::as_str), Some("passed"));
    }

    #[test]
    fn an_accepted_exception_is_listed_with_its_date_and_can_be_retracted() {
        let (root, circuit, ear) = started("excepted");
        std::fs::write(root.join("src/report.ts"), TOTALS.replace("totals", "summarize")).unwrap();
        for _ in 0..=MAX_ROUNDS {
            circuit.answer(&ear, CLOSE, &close());
        }
        accept(&root).unwrap();
        let listed = exceptions(&root);
        assert_eq!(listed.len(), 1);
        assert_eq!((listed[0].rule, listed[0].file.as_str()), (Some(Rule::R1), "src/report.ts"));
        assert!(listed[0].since > 0);
        retract(&root, &listed[0].key).unwrap();
        assert!(exceptions(&root).is_empty());
    }

    #[test]
    fn accepting_a_held_turn_approves_it_and_the_same_finding_stops_blocking() {
        let (root, circuit, ear) = started("accept");
        std::fs::write(root.join("src/report.ts"), TOTALS.replace("totals", "summarize")).unwrap();
        for _ in 0..=MAX_ROUNDS {
            circuit.answer(&ear, CLOSE, &close());
        }
        accept(&root).unwrap();
        let state = State::load(&root);
        assert!(state.held.is_none() && state.approved == state.end);
        std::fs::write(root.join("src/report.ts"), TOTALS.replace("totals", "summarize").replace("limit: number", "limit: number, _: string")).unwrap();
        assert_eq!(circuit.answer(&ear, CLOSE, &close()), json!({}));
    }

    #[test]
    fn a_clean_turn_is_approved_but_not_while_background_work_is_running() {
        let (root, circuit, ear) = started("clean");
        std::fs::write(root.join("src/report.ts"), REUSES).unwrap();
        let running = json!({ "hook_event_name": "Stop", "background_tasks": [{ "id": "t1" }] });
        assert_eq!(circuit.answer(&ear, CLOSE, &running), json!({}));
        let state = State::load(&root);
        assert_ne!(state.approved, state.end);
        assert_eq!(circuit.answer(&ear, CLOSE, &close()), json!({}));
        let state = State::load(&root);
        assert_eq!(state.approved, state.end);
        assert_eq!(ear.stages(), ["pending", "passed"]);
    }

    struct Scripted(Result<Value, String>);

    impl Reviewer for Scripted {
        fn review(&self, prompt: &str) -> Result<super::super::review::Reviewed, String> {
            assert!(prompt.contains("+import { totals }"), "{prompt}");
            assert!(prompt.contains(&format!("write `why` and `fix` in {}.", language::now().tongue())), "{prompt}");
            self.0.clone().map(|answer| super::super::review::Reviewed { answer, cost: 0.002 })
        }
    }

    fn reviewed(name: &str, answer: Result<Value, String>) -> (PathBuf, Circuit, Ear) {
        let (root, keeper) = project(name);
        let circuit = Circuit::new(&root, "s1", keeper, false).reviewed_by(Arc::new(Scripted(answer)));
        let ear = Ear::default();
        circuit.answer(&ear, PROMPT, &json!({ "prompt": "hola" }));
        std::fs::write(root.join("src/report.ts"), REUSES).unwrap();
        (root, circuit, ear)
    }

    #[test]
    fn a_reviewer_that_cannot_answer_holds_the_turn_for_the_person() {
        let (root, circuit, ear) = reviewed("review-down", Err("no network".into()));
        assert_eq!(circuit.answer(&ear, CLOSE, &close()), json!({}));
        assert_eq!(ear.stages(), ["held"]);
        let state = State::load(&root);
        assert_ne!(state.approved, state.end);
    }

    #[test]
    fn a_medium_finding_is_told_and_the_turn_is_approved() {
        let medium = json!({ "findings": [{ "rule": "S4", "file": "src/report.ts", "quote": "totals([], 2)", "why": "The 2 is not asked for.", "fix": "Pass the limit in.", "confidence": "medium" }] });
        let (root, circuit, ear) = reviewed("review-medium", Ok(medium));
        let told = circuit.answer(&ear, CLOSE, &close());
        assert_eq!(decision(&told), "allow", "{told}");
        assert_eq!(ear.stages(), ["reviewed", "passed"]);
        let state = State::load(&root);
        assert_eq!(state.approved, state.end);
    }

    #[test]
    fn a_high_finding_the_person_accepted_before_no_longer_stops() {
        let high = json!({ "findings": [{ "rule": "S4", "file": "src/report.ts", "quote": "totals([], 2)", "why": "The 2 is not asked for.", "fix": "Pass the limit in.", "confidence": "high" }] });
        let (root, circuit, ear) = reviewed("review-high", Ok(high));
        assert_eq!(decision(&circuit.answer(&ear, CLOSE, &close())), "block");
        let key = "S4:src/report.ts:totals([], 2)".to_string();
        state::save_exceptions(&root, &sens_canon::verdict::Exceptions { keys: [key].into() }).unwrap();
        assert_eq!(decision(&circuit.answer(&ear, CLOSE, &close())), "allow");
    }

    #[test]
    fn a_commit_is_recognized_through_git_s_own_options() {
        for yes in ["git commit -m x", "git add -A && git -c user.name=x commit -m y", "git -C sub push origin main", "cd a; git commit"] {
            assert!(commits(yes), "{yes}");
        }
        for no in ["git status", "git log --grep commit", "npm run commit-msg", "gitk"] {
            assert!(!commits(no), "{no}");
        }
    }
}
