use std::path::{Path, PathBuf};
use std::time::Duration;

use sens_agent::chat::Engine;
use sens_bench::trial::{Condition, Plan};
use sens_bench::{report, task, trial};

const USAGE: &str = "sens-bench run --tasks <carpeta> --condition C0[,C1,C2] --out <carpeta> [--reps 3] [--model claude-sonnet-5-5] [--effort medium] [--only <tarea>] [--minutes 30]
sens-bench report <carpeta>";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(error) = go(&args) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn go(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("run") => run(&args[1..]),
        Some("report") => summarize(Path::new(args.get(1).ok_or(USAGE)?)).map(|text| println!("{text}")),
        _ => Err(USAGE.into()),
    }
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find(|pair| pair[0] == name).map(|pair| pair[1].clone())
}

fn number(args: &[String], name: &str, default: u64) -> Result<u64, String> {
    flag(args, name).map_or(Ok(default), |value| value.parse().map_err(|_| format!("{name} espera un número, no {value}")))
}

fn jscpd() -> Result<PathBuf, String> {
    let found = std::env::var_os("SENS_JSCPD")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../node_modules/jscpd/run-jscpd.js"));
    found.is_file().then_some(found).ok_or_else(|| "falta jscpd: ejecuta npm ci en la raíz del repositorio".into())
}

fn run(args: &[String]) -> Result<(), String> {
    let tasks = task::all(Path::new(&flag(args, "--tasks").ok_or(USAGE)?), flag(args, "--only").as_deref())?;
    if tasks.is_empty() {
        return Err("no hay tareas".into());
    }
    let conditions: Vec<Condition> = flag(args, "--condition").ok_or(USAGE)?.split(',').map(str::parse).collect::<Result<_, _>>()?;
    let out = PathBuf::from(flag(args, "--out").ok_or(USAGE)?);
    let reps = number(args, "--reps", 3)? as u32;
    let patience = Duration::from_secs(number(args, "--minutes", 30)? * 60);
    let model = flag(args, "--model").unwrap_or_else(|| "claude-sonnet-5-5".into());
    let effort = flag(args, "--effort").unwrap_or_else(|| "medium".into());
    let jscpd = jscpd()?;
    let scratch = std::env::temp_dir().join("sens-bench").join(out.file_name().unwrap_or_default());

    let engine = Engine::default();
    for rep in 1..=reps {
        for task in &tasks {
            for &condition in &conditions {
                let plan = Plan {
                    task,
                    condition,
                    rep,
                    model: model.clone(),
                    effort: effort.clone(),
                    scratch: scratch.clone(),
                    diffs: out.join("diffs"),
                    jscpd: jscpd.clone(),
                    patience,
                };
                eprintln!("{} {condition:?} #{rep}…", task.id);
                let done = trial::trial(&engine, &plan);
                eprintln!(
                    "  {} · {:+} líneas · {} tokens{}",
                    if done.valid() { "válida" } else { "no válida" },
                    done.net_lines(),
                    done.tokens(),
                    if done.error.is_empty() { String::new() } else { format!(" · {}", done.error.lines().next().unwrap_or_default()) }
                );
                report::append(&out, &done)?;
            }
        }
    }
    engine.shutdown();
    summarize(&out).map(|text| println!("{text}"))
}

fn summarize(dir: &Path) -> Result<String, String> {
    let text = report::summary(&report::read(dir)?);
    std::fs::write(dir.join(report::SUMMARY), &text).map_err(|error| error.to_string())?;
    Ok(text)
}
