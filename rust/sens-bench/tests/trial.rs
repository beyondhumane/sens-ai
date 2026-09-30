use std::path::{Path, PathBuf};
use std::time::Duration;

use sens_agent::chat::Engine;
use sens_bench::task::Task;
use sens_bench::trial::{self, Condition, Plan};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn fake_engine() -> Engine {
    let fake = root().join("../sens-agent/tests/fixtures/fake-claude.mjs");
    Engine::launching(vec!["node".into(), fake.to_string_lossy().into_owned()])
}

fn tiny_task(base: &Path) -> Task {
    let dir = base.join("saludo");
    std::fs::create_dir_all(dir.join("repo")).unwrap();
    std::fs::create_dir_all(dir.join("accept")).unwrap();
    std::fs::write(dir.join("repo").join("a.txt"), "hola\n").unwrap();
    std::fs::write(dir.join("accept").join("check.mjs"), "import { readFileSync } from 'node:fs';\nprocess.exit(readFileSync('a.txt', 'utf8').includes('hola') ? 0 : 1);\n").unwrap();
    Task {
        id: "saludo".into(),
        dir,
        prompt: "hola".into(),
        language: "text".into(),
        setup: String::new(),
        accept: "node accept/check.mjs".into(),
        check: String::new(),
        format: String::new(),
        reuse: vec!["hola".into()],
        allow: Vec::new(),
        base: None,
        accept_into: "accept".into(),
    }
}

#[test]
fn a_trial_prepares_the_repo_drives_the_engine_and_measures_what_changed() {
    let base = std::env::temp_dir().join("sens-bench-trial");
    let _ = std::fs::remove_dir_all(&base);
    let task = tiny_task(&base);
    let engine = fake_engine();
    let plan = Plan {
        task: &task,
        condition: Condition::C1,
        rep: 1,
        model: "claude-sonnet-5-5".into(),
        effort: "medium".into(),
        scratch: base.join("scratch"),
        diffs: base.join("out").join("diffs"),
        jscpd: root().join("../../node_modules/jscpd/run-jscpd.js"),
        patience: Duration::from_secs(60),
        variant: String::new(),
    };

    let run = trial::trial(&engine, &plan);
    engine.shutdown();

    assert!(run.error.is_empty(), "{}", run.error);
    assert!(run.finished && run.accepted && !run.regressed && run.valid(), "{run:?}");
    assert_eq!(run.canon, sens_canon::VERSION);
    assert_eq!((run.lines_added, run.lines_removed), (0, 0));
    assert!(run.files_added.is_empty() && run.reused.is_empty());
    assert!(run.tokens() > 0.0);
    assert!(plan.diffs.join("saludo-C1-1.diff").is_file());
    assert!(Path::new(&run.folder).join(".git").is_dir());
}
