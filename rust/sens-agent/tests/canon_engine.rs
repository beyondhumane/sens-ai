use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sens_agent::canon::{circuit, state};
use sens_agent::chat::{Decision, Engine, Event, Message, Settings, Sink};
use sens_agent::session;
use sens_canon::verdict::Rule;

type Heard = Arc<Mutex<Vec<Event>>>;

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

fn project(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join("sens-canon-engine").join(name);
    let _ = std::fs::remove_dir_all(&root);
    let files = [
        ("package.json", r#"{ "main": "src/index.ts", "dependencies": { "dayjs": "1" } }"#),
        ("src/index.ts", "import { totals } from './lib/totals.ts';\ntotals([], 1);\n"),
        ("src/lib/totals.ts", TOTALS),
    ];
    for (path, content) in files {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    root
}

fn engine(root: &Path) -> Engine {
    let fake = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("fake-claude.mjs");
    let engine = Engine::launching(vec!["node".into(), fake.to_string_lossy().into_owned()]);
    engine.keeper().ready(root, Duration::from_secs(30)).expect("the project is indexed");
    engine
}

fn ear() -> (Heard, Sink) {
    let heard: Heard = Arc::default();
    let kept = heard.clone();
    let sink: Sink = Arc::new(move |_, event| kept.lock().unwrap().push(event.clone()));
    (heard, sink)
}

fn settings() -> Settings {
    Settings { model: "claude-sonnet-5".into(), effort: "high".into(), mode: "bypassPermissions".into(), ..Settings::default() }
}

fn wait_for(heard: &Heard, what: impl Fn(&Event) -> bool) -> Event {
    let until = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(found) = heard.lock().unwrap().iter().find(|event| what(event)) {
            return found.clone();
        }
        assert!(Instant::now() < until, "no llegó; oído: {:?}", heard.lock().unwrap());
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn run(name: &str, words: &str) -> (PathBuf, Engine, String, Heard) {
    let root = project(name);
    let engine = engine(&root);
    let id = session::open(&root).unwrap();
    let (heard, sink) = ear();
    engine.send(&root, &id, &Message { text: words.into(), ..Message::default() }, settings(), sink).unwrap();
    (root, engine, id, heard)
}

fn finished(heard: &Heard) {
    wait_for(heard, |event| matches!(event, Event::Finished { .. }));
}

fn said(heard: &Heard) -> Vec<String> {
    heard.lock().unwrap().iter().filter_map(|event| match event { Event::Said { text } => Some(text.clone()), _ => None }).collect()
}

fn stages(heard: &Heard) -> Vec<String> {
    heard.lock().unwrap().iter().filter_map(|event| match event { Event::Canon { stage, .. } => Some(stage.clone()), _ => None }).collect()
}

#[test]
fn a_copy_is_denied_before_it_lands_and_the_reuse_passes() {
    let (root, engine, _, heard) = run("copia", "copia");
    finished(&heard);
    assert_eq!(said(&heard), ["copia denegada", "cierre limpio"]);
    let write = heard.lock().unwrap().iter().find_map(|event| match event { Event::Canon { stage, findings, .. } if stage == "write" && !findings.is_empty() => Some(findings.clone()), _ => None }).unwrap();
    assert_eq!(write[0].rule, Rule::R1);
    assert!(std::fs::read_to_string(root.join("src/report.ts")).unwrap().contains("import { totals }"));
    assert!(stages(&heard).contains(&"passed".to_string()));
    engine.shutdown();
}

#[test]
fn what_the_terminal_writes_is_judged_too() {
    let (_, engine, _, heard) = run("terminal", "terminal");
    finished(&heard);
    assert_eq!(said(&heard), ["terminal bloqueada", "cierre limpio"]);
    engine.shutdown();
}

#[test]
fn three_blocked_endings_hold_the_turn_for_the_person() {
    let (root, engine, _, heard) = run("rondas", "rondas");
    finished(&heard);
    assert_eq!(said(&heard), ["ronda 1 bloqueada", "ronda 2 bloqueada", "ronda 3 bloqueada", "ronda 4 libre"]);
    wait_for(&heard, |event| matches!(event, Event::Held { .. }));
    assert_eq!(circuit::held(&root).unwrap()[0].rule, Rule::R1);
    engine.shutdown();
}

#[test]
fn a_commit_of_unapproved_changes_is_denied() {
    let (_, engine, _, heard) = run("commit", "commit");
    finished(&heard);
    assert_eq!(said(&heard)[0], "commit denegado");
    engine.shutdown();
}

#[test]
fn the_agent_s_settings_cannot_be_written_and_a_command_that_writes_them_is_undone() {
    let (_, engine, _, heard) = run("ajustes", "ajustes");
    finished(&heard);
    assert_eq!(said(&heard), ["ajustes denegados", "ajustes devueltos"]);
    engine.shutdown();
}

#[test]
fn a_new_dependency_waits_for_the_person() {
    for (allow, expected) in [(false, "dependencia denegada"), (true, "dependencia aceptada")] {
        let (root, engine, id, heard) = run(if allow { "dependencia-si" } else { "dependencia-no" }, "dependencia");
        let Event::Asking { request, tool, .. } = wait_for(&heard, |event| matches!(event, Event::Asking { .. })) else { unreachable!() };
        assert_eq!(tool, "sens.dependency");
        engine.answer(&id, &request, &Decision { allow, ..Decision::default() }).unwrap();
        finished(&heard);
        assert_eq!(said(&heard), [expected]);
        assert_eq!(state::exceptions(&root).keys.contains("R3:moment"), allow);
        engine.shutdown();
    }
}

#[test]
fn a_crash_leaves_the_turn_unapproved_and_the_next_turn_judges_it() {
    let (root, engine, id, heard) = run("muere", "muere copia");
    wait_for(&heard, |event| matches!(event, Event::Failed { .. }));
    assert!(root.join("src/report.ts").exists());
    let before = state::State::load(&root).approved;
    let (again, sink) = ear();
    engine.send(&root, &id, &Message { text: "hola".into(), ..Message::default() }, settings(), sink).unwrap();
    finished(&again);
    assert!(stages(&again).contains(&"blocked".to_string()), "{:?}", stages(&again));
    assert_eq!(state::State::load(&root).approved, before);
    engine.shutdown();
}

#[test]
fn a_turn_the_circuit_never_heard_is_flagged() {
    let (_, engine, _, heard) = run("sordo", "sordo");
    finished(&heard);
    assert!(stages(&heard).contains(&"detached".to_string()), "{:?}", stages(&heard));
    engine.shutdown();
}
