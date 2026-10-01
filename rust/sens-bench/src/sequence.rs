use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use regex::Regex;
use sens_agent::chat::Engine;
use sens_canon::verdict::Finding;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::project::{self, Probe, Shape};
use crate::report;
use crate::stats::{self, Estimate};
use crate::trial::{self, Condition};
use crate::{drive, git, measure, shell};

pub const STEPS: &str = "steps.jsonl";
pub const SUMMARY: &str = "summary.md";
const SKIPPED: [&str; 3] = ["node_modules", ".git", ".sens"];

#[derive(Deserialize)]
struct Declared {
    #[serde(default)]
    setup: String,
    check: String,
    accept: String,
    accept_into: String,
    source: String,
    #[serde(default)]
    probes: BTreeMap<String, String>,
}

pub struct Sequence {
    pub id: String,
    pub dir: PathBuf,
    pub setup: String,
    pub check: String,
    pub accept: String,
    pub accept_into: String,
    pub source: String,
    pub probes: Vec<Probe>,
    pub steps: Vec<Step>,
}

pub struct Step {
    pub id: String,
    pub dir: PathBuf,
    pub prompt: String,
}

impl Step {
    fn hidden(&self) -> Result<Vec<String>, String> {
        let folder = self.dir.join("accept");
        let mut names: Vec<String> = std::fs::read_dir(&folder)
            .map_err(|error| format!("{}: {error}", folder.display()))?
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        Ok(names)
    }
}

fn text_of(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn load(dir: &Path) -> Result<Sequence, String> {
    let dir = std::path::absolute(dir).unwrap_or_else(|_| dir.to_path_buf());
    let declared: Declared = toml::from_str(&text_of(&dir.join("sequence.toml"))?).map_err(|error| format!("sequence.toml: {error}"))?;
    let probes = declared
        .probes
        .into_iter()
        .map(|(name, pattern)| Regex::new(&pattern).map(|pattern| Probe { name: name.clone(), pattern }).map_err(|error| format!("sonda {name}: {error}")))
        .collect::<Result<Vec<_>, _>>()?;
    let tasks = dir.join("tasks");
    let mut folders: Vec<PathBuf> = std::fs::read_dir(&tasks)
        .map_err(|error| format!("{}: {error}", tasks.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|folder| folder.join("prompt.md").is_file())
        .collect();
    folders.sort();
    let steps = folders
        .into_iter()
        .map(|folder| {
            let prompt = text_of(&folder.join("prompt.md"))?.trim().to_string();
            let id = folder.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
            Ok(Step { id, dir: folder, prompt })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Sequence {
        id: dir.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default(),
        dir,
        setup: declared.setup,
        check: declared.check,
        accept: declared.accept,
        accept_into: declared.accept_into,
        source: declared.source,
        probes,
        steps,
    })
}

fn prepare(sequence: &Sequence, work: &Path) -> Result<(), String> {
    let _ = std::fs::remove_dir_all(work);
    trial::copy_without(&sequence.dir.join("base"), work, &SKIPPED)?;
    git::baseline(work)?;
    let setup = shell::run(work, &sequence.setup);
    if !setup.ok {
        return Err(format!("setup: {}", setup.output.trim()));
    }
    Ok(())
}

pub fn passing(report: &str) -> BTreeSet<String> {
    let parsed: Value = serde_json::from_str(report).unwrap_or(Value::Null);
    parsed["testResults"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|file| file["status"] == "passed")
        .filter_map(|file| file["name"].as_str())
        .map(|name| name.replace('\\', "/").rsplit('/').next().unwrap_or_default().to_string())
        .collect()
}

#[derive(Debug, Default)]
pub struct Judged {
    pub check: bool,
    pub passed: BTreeSet<String>,
    pub said: String,
}

fn judged(sequence: &Sequence, work: &Path, upto: usize) -> Result<Judged, String> {
    let hidden = work.join(&sequence.accept_into);
    let _ = std::fs::remove_dir_all(&hidden);
    let check = trial::checked(work, &sequence.check);
    for step in &sequence.steps[..upto] {
        trial::copy(&step.dir.join("accept"), &hidden)?;
    }
    let report = std::env::temp_dir().join(format!("sens-bench-accept-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&report);
    let accepted = shell::run(work, &sequence.accept.replace("{report}", &report.to_string_lossy()));
    let files = passing(&std::fs::read_to_string(&report).unwrap_or_default());
    let _ = std::fs::remove_file(&report);
    let _ = std::fs::remove_dir_all(&hidden);
    let mut passed = BTreeSet::new();
    for step in &sequence.steps[..upto] {
        let names = step.hidden()?;
        if !names.is_empty() && names.iter().all(|name| files.contains(name)) {
            passed.insert(step.id.clone());
        }
    }
    Ok(Judged { check: check.ok, passed, said: if check.ok { accepted.output } else { check.output } })
}

#[derive(Debug, Default, PartialEq)]
pub struct Checked {
    pub step: String,
    pub failed_before: bool,
    pub check: bool,
    pub passed: bool,
    pub kept: bool,
    pub said: String,
}

impl Checked {
    pub fn valid(&self) -> bool {
        self.failed_before && self.check && self.passed && self.kept
    }
}

pub fn validate(sequence: &Sequence, work: &Path) -> Result<Vec<Checked>, String> {
    prepare(sequence, work)?;
    let mut found = Vec::new();
    for (at, step) in sequence.steps.iter().enumerate() {
        let before = judged(sequence, work, at + 1)?;
        git::git(work, &["apply", "--whitespace=nowarn", &step.dir.join("reference.patch").to_string_lossy()])?;
        let after = judged(sequence, work, at + 1)?;
        git::git(work, &["add", "-A"])?;
        git::git(work, &["commit", "-q", "--allow-empty", "-m", &step.id])?;
        found.push(Checked {
            step: step.id.clone(),
            failed_before: !before.passed.contains(&step.id),
            check: after.check,
            passed: after.passed.contains(&step.id),
            kept: sequence.steps[..at].iter().all(|earlier| after.passed.contains(&earlier.id)),
            said: after.said,
        });
    }
    Ok(found)
}

pub struct Plan<'a> {
    pub sequence: &'a Sequence,
    pub condition: Condition,
    pub rep: u32,
    pub model: String,
    pub effort: String,
    pub scratch: PathBuf,
    pub out: PathBuf,
    pub jscpd: PathBuf,
    pub patience: Duration,
}

impl Plan<'_> {
    pub fn name(&self) -> String {
        folder(&self.sequence.id, self.condition, self.rep)
    }
}

fn folder(sequence: &str, condition: Condition, rep: u32) -> String {
    format!("{sequence}-{condition:?}-{rep}")
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StepRun {
    pub sequence: String,
    pub condition: Option<Condition>,
    pub rep: u32,
    pub step: u32,
    pub task: String,
    pub model: String,
    pub effort: String,
    pub canon: String,
    pub error: String,
    pub finished: bool,
    pub millis: u64,
    pub turns: u64,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub held: bool,
    pub check: bool,
    pub accepted: bool,
    pub broken: Vec<String>,
    pub lines_added: u64,
    pub lines_removed: u64,
    pub shape: Shape,
    pub circuit: Vec<String>,
    pub findings: Vec<Finding>,
    pub commit: String,
}

impl StepRun {
    pub fn tokens(&self) -> f64 {
        (self.tokens_in + self.tokens_out) as f64
    }
}

pub fn step(engine: &Engine, plan: &Plan, at: usize, from: Option<&str>) -> Result<StepRun, String> {
    let task = &plan.sequence.steps[at];
    let work = plan.scratch.join(plan.name());
    match from {
        None => prepare(plan.sequence, &work)?,
        Some(commit) => {
            git::git(&work, &["reset", "-q", "--hard", commit])?;
            git::git(&work, &["clean", "-fdq"])?;
        }
    }
    let mut run = StepRun {
        sequence: plan.sequence.id.clone(),
        condition: Some(plan.condition),
        rep: plan.rep,
        step: at as u32 + 1,
        task: task.id.clone(),
        model: plan.model.clone(),
        effort: plan.effort.clone(),
        canon: match plan.condition {
            Condition::C0 => String::new(),
            _ => sens_canon::VERSION.into(),
        },
        ..StepRun::default()
    };
    match drive::drive(engine, &work, &task.prompt, trial::settings(&plan.model, &plan.effort, plan.condition, BTreeMap::new()), plan.patience, &[]) {
        Ok(turn) => {
            run.finished = turn.finished;
            run.millis = turn.millis;
            run.turns = turn.turns;
            run.tokens_in = turn.tokens_in;
            run.tokens_out = turn.tokens_out;
            run.held = turn.held;
            run.circuit = turn.circuit;
            run.findings = turn.findings;
            run.error = turn.error;
        }
        Err(error) => run.error = error,
    }
    let changes = git::changes(&work)?;
    let diffs = plan.out.join("diffs");
    std::fs::create_dir_all(&diffs).map_err(|error| error.to_string())?;
    let stem = format!("{}-{:02}", plan.name(), at + 1);
    std::fs::write(diffs.join(format!("{stem}.diff")), &changes.patch).map_err(|error| error.to_string())?;
    let lines = measure::lines(&changes.numstat);
    run.lines_added = lines.added;
    run.lines_removed = lines.removed;
    let judged = judged(plan.sequence, &work, at + 1)?;
    run.check = judged.check;
    run.accepted = judged.passed.contains(&task.id);
    run.broken = plan.sequence.steps[..at].iter().filter(|earlier| !judged.passed.contains(&earlier.id)).map(|earlier| earlier.id.clone()).collect();
    if !(run.check && run.accepted) {
        std::fs::write(diffs.join(format!("{stem}.judged.txt")), &judged.said).map_err(|error| error.to_string())?;
    }
    run.shape = project::shape(&work, &plan.sequence.source, &plan.jscpd, &plan.sequence.probes)?;
    git::git(&work, &["add", "-A"])?;
    git::git(&work, &["commit", "-q", "--allow-empty", "-m", &task.id])?;
    run.commit = git::git(&work, &["rev-parse", "HEAD"])?.trim().to_string();
    Ok(run)
}

pub fn read(out: &Path) -> Result<Vec<StepRun>, String> {
    if !out.join(STEPS).is_file() {
        return Ok(Vec::new());
    }
    report::lines_of(&out.join(STEPS))
}

pub fn append(out: &Path, run: &StepRun) -> Result<(), String> {
    report::add_line(&out.join(STEPS), run)
}

pub fn remeasure(sequence: &Sequence, runs: &mut [StepRun], scratch: &Path, jscpd: &Path) -> Result<(), String> {
    let snapshot = scratch.join("remeasured");
    for run in runs.iter_mut().filter(|run| !run.commit.is_empty()) {
        let Some(condition) = run.condition else {
            continue;
        };
        let _ = std::fs::remove_dir_all(&snapshot);
        git::export(&scratch.join(folder(&run.sequence, condition, run.rep)), &run.commit, &snapshot)?;
        run.shape = project::shape(&snapshot, &sequence.source, jscpd, &sequence.probes)?;
    }
    let _ = std::fs::remove_dir_all(&snapshot);
    Ok(())
}

pub fn rewrite(out: &Path, runs: &[StepRun]) -> Result<(), String> {
    report::write_lines(&out.join(STEPS), runs)
}

pub fn last_commits(runs: &[StepRun]) -> BTreeMap<(Condition, u32), (u32, String)> {
    let mut last: BTreeMap<(Condition, u32), (u32, String)> = BTreeMap::new();
    for run in runs {
        let Some(condition) = run.condition else {
            continue;
        };
        let slot = last.entry((condition, run.rep)).or_insert((0, String::new()));
        if run.step > slot.0 {
            *slot = (run.step, run.commit.clone());
        }
    }
    last
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn slope(points: &[(f64, f64)]) -> Option<f64> {
    let n = points.len() as f64;
    if points.len() < 2 {
        return None;
    }
    let (mx, my) = (points.iter().map(|point| point.0).sum::<f64>() / n, points.iter().map(|point| point.1).sum::<f64>() / n);
    let spread: f64 = points.iter().map(|point| (point.0 - mx).powi(2)).sum();
    (spread > 0.0).then(|| points.iter().map(|point| (point.0 - mx) * (point.1 - my)).sum::<f64>() / spread)
}

fn shown(value: Option<f64>) -> String {
    value.map_or("—".into(), |value| format!("{value:.0}"))
}

fn estimated(value: Option<Estimate>) -> String {
    value.map_or("—".into(), |value| format!("{:+.1} [{:+.1}, {:+.1}]", value.point, value.low, value.high))
}

type Measure = (String, Box<dyn Fn(&StepRun) -> f64>);

fn measures(runs: &[StepRun]) -> Vec<Measure> {
    let mut list: Vec<Measure> = vec![
        ("Líneas duplicadas (jscpd)".into(), Box::new(|run: &StepRun| run.shape.duplicated as f64)),
        ("Funciones con casi-copia (Sens)".into(), Box::new(|run: &StepRun| run.shape.copies as f64)),
        ("Líneas de código".into(), Box::new(|run: &StepRun| run.shape.lines as f64)),
        ("Funciones".into(), Box::new(|run: &StepRun| run.shape.functions as f64)),
        ("Código muerto".into(), Box::new(|run: &StepRun| run.shape.dead as f64)),
        ("Tests ocultos rotos".into(), Box::new(|run: &StepRun| run.broken.len() as f64)),
    ];
    let probes: BTreeSet<String> = runs.iter().flat_map(|run| run.shape.probes.keys().cloned()).collect();
    for probe in probes {
        let key = probe.clone();
        list.push((format!("Sitios que hacen «{probe}»"), Box::new(move |run: &StepRun| run.shape.probes.get(&key).copied().unwrap_or(0) as f64)));
    }
    list
}

pub fn summary(runs: &[StepRun]) -> String {
    let mut out = String::new();
    let conditions: Vec<Condition> = runs.iter().filter_map(|run| run.condition).collect::<BTreeSet<_>>().into_iter().collect();
    let reps: BTreeSet<u32> = runs.iter().map(|run| run.rep).collect();
    let model = runs.first().map(|run| format!("{} · {}", run.model, run.effort)).unwrap_or_default();
    let _ = writeln!(out, "# Horizonte\n\nModelo: {model}. Pasos registrados: {}. Secuencias por brazo: {}.\n", runs.len(), reps.len());
    let cell = |condition: Condition, step: u32| -> Vec<&StepRun> { runs.iter().filter(|run| run.condition == Some(condition) && run.step == step).collect() };
    let steps: BTreeSet<u32> = runs.iter().map(|run| run.step).collect();

    let _ = writeln!(out, "## Paso a paso\n\nMedia de las secuencias de cada brazo, sobre el proyecto entero después de cada paso.\n");
    let mut header = String::from("| Paso | Tarea |");
    let mut rule = String::from("| --- | --- |");
    for condition in &conditions {
        let _ = write!(header, " {condition:?} aceptadas | {condition:?} duplicadas | {condition:?} líneas | {condition:?} tokens |");
        rule.push_str(" --- | --- | --- | --- |");
    }
    let _ = writeln!(out, "{header}\n{rule}");
    for &step in &steps {
        let task = runs.iter().find(|run| run.step == step).map(|run| run.task.clone()).unwrap_or_default();
        let mut row = format!("| {step} | {task} |");
        for &condition in &conditions {
            let here = cell(condition, step);
            let average = |metric: fn(&StepRun) -> f64| shown(mean(&here.iter().map(|run| metric(run)).collect::<Vec<_>>()));
            let _ = write!(
                row,
                " {}/{} | {} | {} | {} |",
                here.iter().filter(|run| run.accepted && run.check).count(),
                here.len(),
                average(|run| run.shape.duplicated as f64),
                average(|run| run.shape.lines as f64),
                average(StepRun::tokens)
            );
        }
        let _ = writeln!(out, "{row}");
    }

    let reached: Option<u32> = conditions
        .iter()
        .flat_map(|&condition| reps.iter().map(move |&rep| (condition, rep)))
        .map(|(condition, rep)| runs.iter().filter(|run| run.condition == Some(condition) && run.rep == rep).map(|run| run.step).max().unwrap_or(0))
        .min();
    if let (Some(last), Some(base)) = (reached.filter(|&last| last > 0), conditions.first().copied()) {
        let _ = writeln!(out, "\n## En el paso {last}\n\nDiferencia de medianas frente a {base:?}, con intervalo al 95 % por bootstrap sobre las secuencias.\n");
        let others: Vec<Condition> = conditions.iter().copied().filter(|&condition| condition != base).collect();
        let mut header = format!("| Medida | {base:?} |");
        let mut rule = String::from("| --- | --- |");
        for other in &others {
            let _ = write!(header, " {other:?} | {other:?} − {base:?} |");
            rule.push_str(" --- | --- |");
        }
        let _ = writeln!(out, "{header}\n{rule}");
        for (name, metric) in measures(runs) {
            let values = |condition: Condition| -> Vec<f64> { cell(condition, last).iter().map(|run| metric(run)).collect() };
            let mut row = format!("| {name} | {} |", shown(stats::median(&values(base))));
            for &other in &others {
                let _ = write!(row, " {} | {} |", shown(stats::median(&values(other))), estimated(stats::difference(&values(base), &values(other))));
            }
            let _ = writeln!(out, "{row}");
        }
        let accepted = |condition: Condition| runs.iter().filter(|run| run.condition == Some(condition) && run.step <= last && run.accepted && run.check).count();
        let tried = |condition: Condition| runs.iter().filter(|run| run.condition == Some(condition) && run.step <= last).count();
        let _ = writeln!(out, "\nTareas aceptadas hasta el paso {last}: {}.", conditions.iter().map(|&condition| format!("{condition:?} {}/{}", accepted(condition), tried(condition))).collect::<Vec<_>>().join(" · "));

        let slopes = |condition: Condition| -> Vec<f64> {
            reps.iter()
                .filter_map(|&rep| slope(&runs.iter().filter(|run| run.condition == Some(condition) && run.rep == rep && run.step <= last).map(|run| (run.step as f64, run.tokens())).collect::<Vec<_>>()))
                .collect()
        };
        let _ = writeln!(out, "\n## Coste por tarea\n\nPendiente de los tokens de cada tarea a lo largo de la secuencia (tokens más por cada tarea que pasa).\n");
        let _ = writeln!(out, "| Brazo | Pendiente | Frente a {base:?} |\n| --- | --- | --- |");
        for &condition in &conditions {
            let versus = if condition == base { "—".into() } else { estimated(stats::difference(&slopes(base), &slopes(condition))) };
            let _ = writeln!(out, "| {condition:?} | {} | {versus} |", shown(mean(&slopes(condition))));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(condition: Condition, rep: u32, step: u32, duplicated: u64, tokens: u64) -> StepRun {
        StepRun {
            condition: Some(condition),
            rep,
            step,
            task: format!("{step:02}-t"),
            accepted: true,
            check: true,
            tokens_in: tokens,
            commit: format!("c{step}"),
            shape: Shape { duplicated, probes: BTreeMap::from([("dinero".into(), 1)]), ..Shape::default() },
            ..StepRun::default()
        }
    }

    #[test]
    fn a_vitest_report_says_which_files_passed() {
        let report = r#"{ "testResults": [ { "name": "C:\\w\\tests\\_accept\\t01.test.ts", "status": "passed" }, { "name": "/w/tests/_accept/t02.test.ts", "status": "failed" } ] }"#;
        assert_eq!(passing(report), BTreeSet::from(["t01.test.ts".to_string()]));
        assert!(passing("not json").is_empty());
    }

    #[test]
    fn a_batch_resumes_each_arm_from_its_last_step() {
        let runs = [run(Condition::C0, 1, 1, 0, 1), run(Condition::C0, 1, 2, 0, 1), run(Condition::C2, 1, 1, 0, 1)];
        let last = last_commits(&runs);
        assert_eq!(last[&(Condition::C0, 1)], (2, "c2".to_string()));
        assert_eq!(last[&(Condition::C2, 1)], (1, "c1".to_string()));
    }

    #[test]
    fn the_cost_of_a_task_grows_by_the_slope_of_its_tokens() {
        assert_eq!(slope(&[(1.0, 10.0), (2.0, 20.0), (3.0, 30.0)]), Some(10.0));
        assert_eq!(slope(&[(1.0, 10.0)]), None);
    }

    #[test]
    fn the_summary_follows_each_arm_step_by_step_and_compares_where_all_arrived() {
        let mut runs = Vec::new();
        for rep in 1..=2 {
            for step in 1..=3 {
                runs.push(run(Condition::C0, rep, step, 10 * step as u64, 100 * step as u64));
                runs.push(run(Condition::C2, rep, step, step as u64, 50));
            }
        }
        runs.push(run(Condition::C2, 1, 4, 0, 50));
        let text = summary(&runs);
        assert!(text.contains("| 3 | 03-t | 2/2 | 30 | 0 | 300 | 2/2 | 3 | 0 | 50 |"), "{text}");
        assert!(text.contains("## En el paso 3"), "{text}");
        assert!(text.contains("| Líneas duplicadas (jscpd) | 30 | 3 | -27.0 ["), "{text}");
        assert!(text.contains("Sitios que hacen «dinero»"), "{text}");
        assert!(text.contains("| C0 | 100 | — |"), "{text}");
        assert!(text.contains("| C2 | 0 | -100.0 ["), "{text}");
    }
}
