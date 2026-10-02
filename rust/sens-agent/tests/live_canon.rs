use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sens_agent::chat::{Engine, Event, Message, Settings, Sink};
use sens_agent::session;
use sens_canon::verdict::Rule;

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

fn project() -> PathBuf {
    let root = std::env::temp_dir().join("sens-live-canon");
    let _ = std::fs::remove_dir_all(&root);
    for (path, content) in [("package.json", r#"{ "main": "src/index.ts" }"#), ("src/index.ts", "import { totals } from './lib/totals.ts';\ntotals([], 1);\n"), ("src/lib/totals.ts", TOTALS)] {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    root
}

#[test]
#[ignore]
fn a_real_claude_code_is_stopped_from_copying_and_the_circuit_hears_every_turn() {
    let root = project();
    let engine = Engine::default();
    engine.keeper().ready(&root, Duration::from_secs(30)).unwrap();
    let id = session::open(&root).unwrap();
    let heard: Arc<Mutex<Vec<Event>>> = Arc::default();
    let kept = heard.clone();
    let sink: Sink = Arc::new(move |_, event| kept.lock().unwrap().push(event.clone()));
    let settings = Settings { model: "haiku".into(), mode: "bypassPermissions".into(), extra: vec!["--safe-mode".into()], ..Settings::default() };
    let words = "Create src/report.ts exporting summarize(rows, limit) that computes exactly what totals in src/lib/totals.ts computes. Write the whole function body in the new file.";
    engine.send(&root, &id, &Message { text: words.into(), ..Message::default() }, settings, sink).unwrap();

    let until = Instant::now() + Duration::from_secs(300);
    while !heard.lock().unwrap().iter().any(|event| matches!(event, Event::Finished { .. } | Event::Failed { .. })) {
        assert!(Instant::now() < until, "no terminó");
        std::thread::sleep(Duration::from_millis(200));
    }
    engine.shutdown();

    let events = heard.lock().unwrap().clone();
    let stages: Vec<&str> = events.iter().filter_map(|event| match event { Event::Canon { stage, .. } => Some(stage.as_str()), _ => None }).collect();
    assert!(!stages.contains(&"detached"), "{stages:?}");
    let copied = events.iter().any(|event| matches!(event, Event::Canon { findings, .. } if findings.iter().any(|finding| finding.rule == Rule::R1)));
    let report = std::fs::read_to_string(root.join("src/report.ts")).unwrap_or_default();
    assert!(copied || report.contains("totals"), "stages {stages:?}, report:\n{report}");
    assert!(!events.iter().any(|event| matches!(event, Event::Held { .. })) || sens_agent::canon::circuit::held(&root).is_some());
}
