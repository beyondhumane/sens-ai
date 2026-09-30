use std::path::{Path, PathBuf};
use std::time::Duration;

use sens_agent::chat::Engine;
use sens_bench::task;
use sens_bench::trial::{self, Condition, Plan};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[test]
#[ignore]
fn every_pilot_task_prepares_measures_and_fails_acceptance_when_nothing_changes() {
    let tasks = task::all(&root().join("../../bench/tasks"), None).unwrap();
    assert_eq!(tasks.len(), 3);
    let fake = root().join("../sens-agent/tests/fixtures/fake-claude.mjs");
    let engine = Engine::launching(vec!["node".into(), fake.to_string_lossy().into_owned()]);
    let base = std::env::temp_dir().join("sens-bench-pilot");
    let _ = std::fs::remove_dir_all(&base);

    for task in &tasks {
        let plan = Plan {
            task,
            condition: Condition::C0,
            rep: 1,
            model: "claude-sonnet-5-5".into(),
            effort: "medium".into(),
            scratch: base.join("scratch"),
            diffs: base.join("diffs"),
            jscpd: root().join("../../node_modules/jscpd/run-jscpd.js"),
            patience: Duration::from_secs(120),
            variant: String::new(),
        };
        let run = trial::trial(&engine, &plan);
        assert!(run.error.is_empty(), "{}: {}", task.id, run.error);
        assert!(run.finished, "{}", task.id);
        assert!(!run.regressed, "{}: the old tests fail on the base", task.id);
        assert!(!run.accepted, "{}: acceptance passes without any change", task.id);
        assert_eq!((run.lines_added, run.lines_removed), (0, 0), "{}", task.id);
    }
    engine.shutdown();
}
