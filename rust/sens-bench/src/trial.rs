use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use sens_agent::chat::{Canon, Engine, Settings};
use serde::{Deserialize, Serialize};

use crate::drive::{self, Turn};
use crate::task::Task;
use crate::{git, measure, shell};

pub const CONDITIONS: [Condition; 3] = [Condition::C0, Condition::C1, Condition::C2];
pub const ISOLATED: &str = "--safe-mode";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Condition {
    C0,
    C1,
    C2,
}

impl Condition {
    pub fn canon(self) -> Canon {
        match self {
            Condition::C0 => Canon::Off,
            Condition::C1 => Canon::Instructions,
            Condition::C2 => Canon::Full,
        }
    }
}

impl FromStr for Condition {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        CONDITIONS
            .into_iter()
            .find(|condition| format!("{condition:?}").eq_ignore_ascii_case(text))
            .ok_or_else(|| format!("condición desconocida: {text} (C0, C1 o C2)"))
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Run {
    pub task: String,
    pub condition: Option<Condition>,
    pub rep: u32,
    pub model: String,
    pub effort: String,
    pub canon: String,
    pub error: String,
    pub finished: bool,
    pub millis: u64,
    pub turns: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub asked: u64,
    pub held: bool,
    pub accepted: bool,
    pub regressed: bool,
    pub lines_added: u64,
    pub lines_removed: u64,
    #[serde(default)]
    pub test_lines_added: u64,
    #[serde(default)]
    pub test_lines_removed: u64,
    pub files_added: Vec<String>,
    pub dependencies_added: Vec<String>,
    pub duplicated_before: u64,
    pub duplicated_after: u64,
    pub reused: Vec<String>,
    #[serde(default)]
    pub planted: Vec<String>,
    pub folder: String,
}

impl Run {
    pub fn net_lines(&self) -> f64 {
        self.lines_added as f64 - self.lines_removed as f64
    }

    pub fn net_code_lines(&self) -> f64 {
        self.net_lines() - self.net_test_lines()
    }

    pub fn net_test_lines(&self) -> f64 {
        self.test_lines_added as f64 - self.test_lines_removed as f64
    }

    pub fn duplication(&self) -> f64 {
        self.duplicated_after as f64 - self.duplicated_before as f64
    }

    pub fn tokens(&self) -> f64 {
        (self.tokens_in + self.tokens_out) as f64
    }

    pub fn valid(&self) -> bool {
        self.error.is_empty() && self.accepted && !self.regressed
    }
}

pub struct Plan<'a> {
    pub task: &'a Task,
    pub condition: Condition,
    pub rep: u32,
    pub model: String,
    pub effort: String,
    pub scratch: PathBuf,
    pub diffs: PathBuf,
    pub jscpd: PathBuf,
    pub patience: Duration,
}

pub fn name(task: &str, condition: Condition, rep: u32) -> String {
    format!("{task}-{condition:?}-{rep}")
}

impl Plan<'_> {
    fn name(&self) -> String {
        name(&self.task.id, self.condition, self.rep)
    }

    fn settings(&self) -> Settings {
        Settings {
            model: self.model.clone(),
            effort: self.effort.clone(),
            mode: "bypassPermissions".into(),
            canon: self.condition.canon(),
            extra: vec![ISOLATED.into()],
            ..Settings::default()
        }
    }
}

pub fn trial(engine: &Engine, plan: &Plan) -> Run {
    let work = plan.scratch.join(plan.name());
    let mut run = Run {
        task: plan.task.id.clone(),
        condition: Some(plan.condition),
        rep: plan.rep,
        model: plan.model.clone(),
        effort: plan.effort.clone(),
        canon: match plan.condition {
            Condition::C0 => String::new(),
            _ => sens_canon::VERSION.into(),
        },
        folder: work.to_string_lossy().into_owned(),
        planted: plan.task.reuse.clone(),
        ..Run::default()
    };
    if let Err(error) = attempt(engine, plan, &work, &mut run) {
        run.error = error;
    }
    run
}

fn attempt(engine: &Engine, plan: &Plan, work: &Path, run: &mut Run) -> Result<(), String> {
    prepare(plan.task, work)?;
    run.duplicated_before = measure::duplicated_lines(work, &plan.jscpd)?;

    let turn: Turn = drive::drive(engine, work, &plan.task.prompt, plan.settings(), plan.patience)?;
    run.finished = turn.finished;
    run.millis = turn.millis;
    run.turns = turn.turns;
    run.tokens_in = turn.tokens_in;
    run.tokens_out = turn.tokens_out;
    run.asked = turn.asked;
    run.held = turn.held;
    if !turn.error.is_empty() {
        run.error = turn.error;
    }

    shell::run(work, &plan.task.format);
    let changes = git::changes(work)?;
    std::fs::create_dir_all(&plan.diffs).map_err(|error| error.to_string())?;
    std::fs::write(plan.diffs.join(format!("{}.diff", plan.name())), &changes.patch).map_err(|error| error.to_string())?;
    let lines = measure::lines(&changes.numstat);
    let statuses = measure::statuses(&changes.statuses);
    let tests = measure::test_lines(&changes.patch, |path| git::at_base(work, path), |path| std::fs::read_to_string(work.join(path)).unwrap_or_default());
    run.lines_added = lines.added;
    run.lines_removed = lines.removed;
    run.test_lines_added = tests.added;
    run.test_lines_removed = tests.removed;
    run.files_added = measure::new_files(&statuses);
    run.dependencies_added = measure::dependencies_added(work, &statuses);
    run.reused = measure::reused(&changes.patch, &plan.task.reuse);
    run.duplicated_after = measure::duplicated_lines(work, &plan.jscpd)?;

    judge(plan.task, work, &plan.diffs, &plan.name(), run)
}

pub fn judge(task: &Task, work: &Path, diffs: &Path, name: &str, run: &mut Run) -> Result<(), String> {
    copy(&task.acceptance(), &work.join("accept"))?;
    run.regressed = !shell::run(work, &task.check).ok;
    let accepted = shell::run(work, &task.accept);
    run.accepted = accepted.ok;
    let kept = diffs.join(format!("{name}.accept.txt"));
    if accepted.ok {
        let _ = std::fs::remove_file(kept);
        return Ok(());
    }
    std::fs::write(kept, &accepted.output).map_err(|error| error.to_string())
}

fn prepare(task: &Task, work: &Path) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(work);
    copy(&task.repo(), work)?;
    let setup = shell::run(work, &task.setup);
    if !setup.ok {
        return Err(format!("setup: {}", setup.output.trim()));
    }
    shell::run(work, &task.format);
    git::baseline(work)
}

pub fn copy(from: &Path, to: &Path) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|error| format!("{}: {error}", to.display()))?;
    for entry in std::fs::read_dir(from).map_err(|error| format!("{}: {error}", from.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let target = to.join(entry.file_name());
        if entry.file_type().map_err(|error| error.to_string())?.is_dir() {
            copy(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(|error| format!("{}: {error}", target.display()))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditions_read_in_any_case_and_map_to_the_canon() {
        assert_eq!("c1".parse::<Condition>().unwrap(), Condition::C1);
        assert!("C3".parse::<Condition>().is_err());
        assert_eq!(Condition::C0.canon(), Canon::Off);
        assert_eq!(Condition::C2.canon(), Canon::Full);
    }

    #[test]
    fn every_condition_runs_claude_code_without_the_person_s_own_customizations() {
        let task = crate::task::Task {
            id: "t".into(),
            dir: PathBuf::new(),
            prompt: String::new(),
            language: String::new(),
            setup: String::new(),
            accept: String::new(),
            check: String::new(),
            format: String::new(),
            reuse: Vec::new(),
            allow: Vec::new(),
        };
        for condition in CONDITIONS {
            let plan = Plan { task: &task, condition, rep: 1, model: "m".into(), effort: "e".into(), scratch: PathBuf::new(), diffs: PathBuf::new(), jscpd: PathBuf::new(), patience: Duration::ZERO };
            let settings = plan.settings();
            assert_eq!(settings.extra, [ISOLATED]);
            assert_eq!(settings.canon, condition.canon());
            assert_eq!(settings.mode, "bypassPermissions");
        }
    }

    #[test]
    fn only_accepted_runs_without_regressions_or_errors_are_valid() {
        let good = Run { accepted: true, ..Run::default() };
        assert!(good.valid());
        assert!(!Run { regressed: true, ..good.clone() }.valid());
        assert!(!Run { error: "x".into(), ..good.clone() }.valid());
        assert!(!Run::default().valid());
    }
}
