use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::Path;

use crate::stats::{self, Estimate};
use crate::trial::{CONDITIONS, Condition, Run};

pub const RUNS: &str = "runs.jsonl";
pub const SUMMARY: &str = "summary.md";

type Metric = (&'static str, fn(&Run) -> f64);

const COMPARED: [Metric; 3] = [("Líneas netas", Run::net_lines), ("Duplicación añadida", Run::duplication), ("Tokens", Run::tokens)];

pub fn read(dir: &Path) -> Result<Vec<Run>, String> {
    let text = std::fs::read_to_string(dir.join(RUNS)).map_err(|error| format!("{}: {error}", dir.join(RUNS).display()))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|error| format!("{RUNS}: {error}")))
        .collect()
}

pub fn append(dir: &Path, run: &Run) -> Result<(), String> {
    use std::io::Write as _;
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(RUNS))
        .map_err(|error| error.to_string())?;
    let line = serde_json::to_string(run).map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())
}

fn cell<'a>(runs: &'a [Run], task: &str, condition: Condition) -> Vec<&'a Run> {
    runs.iter().filter(|run| run.task == task && run.condition == Some(condition)).collect()
}

fn valid(runs: &[&Run], metric: fn(&Run) -> f64) -> Vec<f64> {
    runs.iter().filter(|run| run.valid()).map(|run| metric(run)).collect()
}

fn number(value: Option<f64>) -> String {
    value.map_or("—".into(), |value| format!("{value:.0}"))
}

fn estimate(value: Option<Estimate>) -> String {
    value.map_or("—".into(), |value| format!("{:+.0} [{:+.0}, {:+.0}]", value.point, value.low, value.high))
}

pub fn summary(runs: &[Run]) -> String {
    let tasks: BTreeSet<&str> = runs.iter().map(|run| run.task.as_str()).collect();
    let models: BTreeSet<String> = runs.iter().map(|run| format!("{} · {}", run.model, run.effort)).collect();
    let mut out = String::new();
    let _ = writeln!(out, "# SensBench\n");
    let _ = writeln!(out, "Modelo: {}. Ejecuciones: {}.\n", models.into_iter().collect::<Vec<_>>().join(", "), runs.len());
    let _ = writeln!(out, "Las medianas y diferencias usan solo las ejecuciones válidas: aceptadas, sin regresiones y sin error.\n");

    let _ = writeln!(out, "## Por tarea\n");
    let _ = writeln!(out, "| Tarea | Condición | Válidas | Aceptadas | Regresiones | Líneas netas | Ficheros nuevos | Dependencias | Duplicación añadida | Reutilizó | Tokens | Minutos |");
    let _ = writeln!(out, "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for task in &tasks {
        for condition in CONDITIONS {
            let here = cell(runs, task, condition);
            if here.is_empty() {
                continue;
            }
            let count = |test: fn(&Run) -> bool| here.iter().filter(|run| test(run)).count();
            let _ = writeln!(
                out,
                "| {task} | {condition:?} | {}/{} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                count(|run| run.valid()),
                here.len(),
                count(|run| run.accepted),
                count(|run| run.regressed),
                number(stats::median(&valid(&here, Run::net_lines))),
                number(stats::median(&valid(&here, |run| run.files_added.len() as f64))),
                count(|run| !run.dependencies_added.is_empty()),
                number(stats::median(&valid(&here, Run::duplication))),
                count(|run| !run.reused.is_empty()),
                number(stats::median(&valid(&here, Run::tokens))),
                number(stats::median(&valid(&here, |run| run.millis as f64 / 60_000.0))),
            );
        }
    }

    let _ = writeln!(out, "\n## Diferencias frente a C0\n");
    let _ = writeln!(out, "Diferencia de medianas, con intervalo al 95 % por bootstrap ({} remuestreos, semilla fija). «Todas» promedia las diferencias de cada tarea. Con menos de dos ejecuciones válidas en una celda no hay estimación.\n", stats::DRAWS);
    let _ = writeln!(out, "| Métrica | Tarea | C1 − C0 | C2 − C0 |");
    let _ = writeln!(out, "| --- | --- | --- | --- |");
    for (label, metric) in COMPARED {
        let versus = |task: &str, condition: Condition| (valid(&cell(runs, task, Condition::C0), metric), valid(&cell(runs, task, condition), metric));
        for task in &tasks {
            let (c1, c2) = (versus(task, Condition::C1), versus(task, Condition::C2));
            let _ = writeln!(out, "| {label} | {task} | {} | {} |", estimate(stats::difference(&c1.0, &c1.1)), estimate(stats::difference(&c2.0, &c2.1)));
        }
        let all = |condition: Condition| {
            let pairs: Vec<(Vec<f64>, Vec<f64>)> = tasks.iter().map(|task| versus(task, condition)).collect();
            let cells: Vec<(&[f64], &[f64])> = pairs.iter().map(|(base, other)| (base.as_slice(), other.as_slice())).collect();
            estimate(stats::pooled(&cells))
        };
        let _ = writeln!(out, "| {label} | Todas | {} | {} |", all(Condition::C1), all(Condition::C2));
    }

    let failed: Vec<&Run> = runs.iter().filter(|run| !run.error.is_empty()).collect();
    if !failed.is_empty() {
        let _ = writeln!(out, "\n## Errores\n");
        for run in failed {
            let _ = writeln!(out, "- {} {:?} #{}: {}", run.task, run.condition.unwrap_or(Condition::C0), run.rep, run.error.lines().next().unwrap_or_default());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(task: &str, condition: Condition, net: u64, accepted: bool) -> Run {
        Run { task: task.into(), condition: Some(condition), accepted, lines_added: net, model: "m".into(), effort: "e".into(), ..Run::default() }
    }

    #[test]
    fn the_summary_counts_valid_runs_and_compares_against_c0() {
        let runs = vec![
            run("t", Condition::C0, 30, true),
            run("t", Condition::C0, 34, true),
            run("t", Condition::C0, 99, false),
            run("t", Condition::C1, 20, true),
            run("t", Condition::C1, 22, true),
        ];
        let text = summary(&runs);
        assert!(text.contains("| t | C0 | 2/3 | 2 | 0 | 32 |"), "{text}");
        assert!(text.contains("| Líneas netas | t | -11 ["), "{text}");
        assert!(text.contains("| Líneas netas | Todas | -11 ["), "{text}");
    }

    #[test]
    fn runs_survive_a_round_trip_through_the_file() {
        let dir = std::env::temp_dir().join("sens-bench-report");
        let _ = std::fs::remove_dir_all(&dir);
        append(&dir, &run("t", Condition::C2, 5, true)).unwrap();
        append(&dir, &run("u", Condition::C0, 7, false)).unwrap();
        let back = read(&dir).unwrap();
        assert_eq!(back.len(), 2);
        assert_eq!(back[0].condition, Some(Condition::C2));
        assert_eq!(back[1].lines_added, 7);
    }
}
