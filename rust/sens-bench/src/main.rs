use std::path::{Path, PathBuf};
use std::time::Duration;

use sens_agent::chat::Engine;
use sens_bench::trial::{Condition, Plan};
use sens_bench::{measure, report, sequence, task, trial};

const USAGE: &str = "sens-bench run --tasks <carpeta> --condition C0[,C1,C2] --out <carpeta> [--reps 3] [--model claude-sonnet-5-5] [--effort medium] [--only <tarea>[,<tarea>]] [--minutes 30] [--variant v2]
sens-bench recheck --tasks <carpeta> <carpeta de resultados>
sens-bench report <carpeta>
sens-bench validate --tasks <carpeta> [--only <tarea>[,<tarea>]]
sens-bench sequence validate <carpeta de la secuencia>
sens-bench sequence run <carpeta de la secuencia> --condition C0[,C2] --out <carpeta> [--reps 3] [--steps 30] [--model claude-sonnet-5-5] [--effort medium] [--minutes 30]
sens-bench sequence report <carpeta>
sens-bench sequence remeasure <carpeta de la secuencia> <carpeta>";

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
        Some("recheck") => recheck(&args[1..]),
        Some("report") => summarize(Path::new(args.get(1).ok_or(USAGE)?)).map(|text| println!("{text}")),
        Some("validate") => validate(&args[1..]),
        Some("sequence") => match args.get(1).map(String::as_str) {
            Some("validate") => validate_sequence(&args[2..]),
            Some("run") => run_sequence(&args[2..]),
            Some("report") => summarize_sequence(Path::new(args.get(2).ok_or(USAGE)?)).map(|text| println!("{text}")),
            Some("remeasure") => remeasure_sequence(&args[2..]),
            _ => Err(USAGE.into()),
        },
        _ => Err(USAGE.into()),
    }
}

fn tail(text: &str) -> String {
    text.lines().rev().take(25).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n")
}

fn mark(ok: bool) -> &'static str {
    if ok { "sí" } else { "no" }
}

fn validate(args: &[String]) -> Result<(), String> {
    let tasks = task::all(Path::new(&flag(args, "--tasks").ok_or(USAGE)?), flag(args, "--only").as_deref())?;
    let scratch = std::env::temp_dir().join("sens-bench-validate");
    let mut broken = 0;
    for task in &tasks {
        let found = trial::validate(task, &scratch.join(&task.id))?;
        println!(
            "{} · base: tests {} · oculta falla {} · referencia: tests {} · oculta pasa {} · {}",
            task.id,
            mark(found.base_check),
            mark(!found.base_accept),
            mark(found.fixed_check),
            mark(found.fixed_accept),
            if found.valid() { "válida" } else { "NO VÁLIDA" }
        );
        if !found.valid() {
            broken += 1;
            println!("{}", tail(&found.said));
        }
    }
    if broken > 0 {
        return Err(format!("{broken} tareas no válidas"));
    }
    Ok(())
}

fn validate_sequence(args: &[String]) -> Result<(), String> {
    let found = sequence::load(Path::new(args.first().ok_or(USAGE)?))?;
    let work = std::env::temp_dir().join("sens-bench-validate").join(&found.id);
    let checked = sequence::validate(&found, &work)?;
    let mut broken = 0;
    for step in &checked {
        println!(
            "{} · el oculto falla antes {} · tests {} · el oculto pasa {} · los anteriores siguen {} · {}",
            step.step,
            mark(step.failed_before),
            mark(step.check),
            mark(step.passed),
            mark(step.kept),
            if step.valid() { "válido" } else { "NO VÁLIDO" }
        );
        if !step.valid() {
            broken += 1;
            println!("{}", tail(&step.said));
        }
    }
    if broken > 0 {
        return Err(format!("{broken} pasos no válidos"));
    }
    let end = sens_bench::project::shape(&work, &found.source, &jscpd()?, &found.probes)?;
    println!(
        "la referencia termina con {} líneas, {} funciones, {} duplicadas, {} casi-copias y {} sin usar · {}",
        end.lines,
        end.functions,
        end.duplicated,
        end.copies,
        end.dead,
        end.probes.iter().map(|(name, count)| format!("{name} {count}")).collect::<Vec<_>>().join(" · ")
    );
    Ok(())
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

struct Batch {
    conditions: Vec<Condition>,
    out: PathBuf,
    reps: u32,
    patience: Duration,
    model: String,
    effort: String,
    jscpd: PathBuf,
    scratch: PathBuf,
}

fn batch(args: &[String]) -> Result<Batch, String> {
    let out = PathBuf::from(flag(args, "--out").ok_or(USAGE)?);
    Ok(Batch {
        conditions: flag(args, "--condition").ok_or(USAGE)?.split(',').map(str::parse).collect::<Result<_, _>>()?,
        reps: number(args, "--reps", 3)? as u32,
        patience: Duration::from_secs(number(args, "--minutes", 30)? * 60),
        model: flag(args, "--model").unwrap_or_else(|| "claude-sonnet-5-5".into()),
        effort: flag(args, "--effort").unwrap_or_else(|| "medium".into()),
        jscpd: jscpd()?,
        scratch: scratch_of(&out),
        out,
    })
}

fn scratch_of(out: &Path) -> PathBuf {
    std::env::temp_dir().join("sens-bench").join(out.file_name().unwrap_or_default())
}

fn remeasure_sequence(args: &[String]) -> Result<(), String> {
    let found = sequence::load(Path::new(args.first().ok_or(USAGE)?))?;
    let out = PathBuf::from(args.get(1).ok_or(USAGE)?);
    let mut runs = sequence::read(&out)?;
    sequence::remeasure(&found, &mut runs, &scratch_of(&out), &jscpd()?)?;
    sequence::rewrite(&out, &runs)?;
    summarize_sequence(&out).map(|text| println!("{text}"))
}

fn said_error(error: &str) -> String {
    if error.is_empty() { String::new() } else { format!(" · {}", error.lines().next().unwrap_or_default()) }
}

fn run(args: &[String]) -> Result<(), String> {
    let tasks = task::all(Path::new(&flag(args, "--tasks").ok_or(USAGE)?), flag(args, "--only").as_deref())?;
    if tasks.is_empty() {
        return Err("no hay tareas".into());
    }
    let batch = batch(args)?;
    let variant = flag(args, "--variant").unwrap_or_default();
    let recorded = report::recorded(&batch.out)?;
    let engine = Engine::default();
    for rep in 1..=batch.reps {
        for task in &tasks {
            for &condition in &batch.conditions {
                if recorded.contains(&trial::name(&task.id, condition, &variant, rep)) {
                    eprintln!("{} {condition:?} #{rep} ya está en {}", task.id, report::RUNS);
                    continue;
                }
                let plan = Plan {
                    task,
                    condition,
                    rep,
                    model: batch.model.clone(),
                    effort: batch.effort.clone(),
                    scratch: batch.scratch.clone(),
                    diffs: batch.out.join("diffs"),
                    jscpd: batch.jscpd.clone(),
                    patience: batch.patience,
                    variant: variant.clone(),
                };
                eprintln!("{} {condition:?} #{rep}…", task.id);
                let done = trial::trial(&engine, &plan);
                eprintln!(
                    "  {} · {:+} líneas de código · {:+} de tests · {} tokens{}",
                    if done.valid() { "válida" } else { "no válida" },
                    done.net_code_lines(),
                    done.net_test_lines(),
                    done.tokens(),
                    said_error(&done.error)
                );
                report::append(&batch.out, &done)?;
            }
        }
    }
    engine.shutdown();
    summarize(&batch.out).map(|text| println!("{text}"))
}

fn run_sequence(args: &[String]) -> Result<(), String> {
    let found = sequence::load(Path::new(args.first().ok_or(USAGE)?))?;
    let batch = batch(args)?;
    let limit = (number(args, "--steps", found.steps.len() as u64)? as usize).min(found.steps.len());
    let mut last = sequence::last_commits(&sequence::read(&batch.out)?);
    let engine = Engine::default();
    for at in 0..limit {
        for rep in 1..=batch.reps {
            for &condition in &batch.conditions {
                let done = last.get(&(condition, rep)).map_or(0, |(step, _)| *step as usize);
                if done > at {
                    continue;
                }
                if done < at {
                    return Err(format!("{condition:?} #{rep} se quedó en el paso {done}: no puede empezar el {}", at + 1));
                }
                let plan = sequence::Plan {
                    sequence: &found,
                    condition,
                    rep,
                    model: batch.model.clone(),
                    effort: batch.effort.clone(),
                    scratch: batch.scratch.clone(),
                    out: batch.out.clone(),
                    jscpd: batch.jscpd.clone(),
                    patience: batch.patience,
                };
                eprintln!("{} {condition:?} #{rep} · paso {} · {}…", found.id, at + 1, found.steps[at].id);
                let from = last.get(&(condition, rep)).map(|(_, commit)| commit.clone());
                let step = sequence::step(&engine, &plan, at, from.as_deref())?;
                eprintln!(
                    "  {} · {} líneas · {} duplicadas · {} tokens{}",
                    if step.accepted && step.check { "aceptada" } else { "no aceptada" },
                    step.shape.lines,
                    step.shape.duplicated,
                    step.tokens(),
                    said_error(&step.error)
                );
                sequence::append(&batch.out, &step)?;
                last.insert((condition, rep), (step.step, step.commit.clone()));
            }
        }
    }
    engine.shutdown();
    summarize_sequence(&batch.out).map(|text| println!("{text}"))
}

fn recheck(args: &[String]) -> Result<(), String> {
    let tasks = task::all(Path::new(&flag(args, "--tasks").ok_or(USAGE)?), None)?;
    let out = PathBuf::from(args.last().ok_or(USAGE)?);
    let jscpd = jscpd()?;
    let mut runs = report::read(&out)?;
    for run in &mut runs {
        let (Some(task), Some(condition)) = (tasks.iter().find(|task| task.id == run.task), run.condition) else {
            continue;
        };
        let folder = PathBuf::from(&run.folder);
        if !folder.is_dir() {
            eprintln!("{} {condition:?} #{}: falta {}", run.task, run.rep, folder.display());
            continue;
        }
        let name = trial::name(&run.task, condition, &run.variant, run.rep);
        run.planted = task.reuse.clone();
        if let Ok(patch) = std::fs::read_to_string(out.join("diffs").join(format!("{name}.diff"))) {
            let tests = measure::test_lines(&patch, |path| sens_bench::git::at_base(&folder, path), |path| std::fs::read_to_string(folder.join(path)).unwrap_or_default());
            run.test_lines_added = tests.added;
            run.test_lines_removed = tests.removed;
            run.duplicated_added = measure::duplicated_added(&folder, &jscpd, &patch)?;
        }
        trial::judge(task, &folder, &out.join("diffs"), &name, run)?;
        eprintln!("{} {condition:?} #{}: {}", run.task, run.rep, if run.valid() { "válida" } else { "no válida" });
    }
    report::rewrite(&out, &runs)?;
    summarize(&out).map(|text| println!("{text}"))
}

fn summarize(dir: &Path) -> Result<String, String> {
    let text = report::summary(&report::read(dir)?);
    std::fs::write(dir.join(report::SUMMARY), &text).map_err(|error| error.to_string())?;
    Ok(text)
}

fn summarize_sequence(dir: &Path) -> Result<String, String> {
    let text = sequence::summary(&sequence::read(dir)?);
    std::fs::write(dir.join(sequence::SUMMARY), &text).map_err(|error| error.to_string())?;
    Ok(text)
}
