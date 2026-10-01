use std::path::{Path, PathBuf};
use std::time::Duration;

use sens_agent::chat::Engine;
use sens_bench::sequence::{self, Plan};
use sens_bench::trial::Condition;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn fake_engine() -> Engine {
    let fake = root().join("../sens-agent/tests/fixtures/fake-claude.mjs");
    Engine::launching(vec!["node".into(), fake.to_string_lossy().into_owned()])
}

const JUDGE: &str = "import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
const results = readdirSync('hidden').map((name) => ({ name: 'hidden/' + name, status: readFileSync('src/hola.ts', 'utf8').includes(readFileSync('hidden/' + name, 'utf8').trim()) ? 'passed' : 'failed' }));
writeFileSync(process.argv[2], JSON.stringify({ testResults: results }));
";

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn tiny_sequence(base: &Path) -> PathBuf {
    let dir = base.join("saludos");
    write(&dir.join("base").join("src").join("hola.ts"), "export function hola(nombre: string) {\n  return `hola ${nombre}`;\n}\n");
    write(&dir.join("base").join(".gitignore"), ".sens/\n");
    write(&dir.join("judge.mjs"), JUDGE);
    let judge = dir.join("judge.mjs").to_string_lossy().replace('\\', "/");
    write(&dir.join("sequence.toml"), &format!("check = \"\"\naccept_into = \"hidden\"\naccept = 'node \"{judge}\" \"{{report}}\"'\nsource = \"src\"\n\n[probes]\nsaludos = 'hola'\n"));
    for (step, wanted) in [("01-hola", "hola"), ("02-nombre", "nombre")] {
        write(&dir.join("tasks").join(step).join("prompt.md"), "hola\n");
        write(&dir.join("tasks").join(step).join("accept").join(format!("{step}.txt")), wanted);
    }
    dir
}

#[test]
fn each_step_starts_where_the_last_one_ended_and_measures_the_whole_project() {
    let base = std::env::temp_dir().join("sens-bench-sequence");
    let _ = std::fs::remove_dir_all(&base);
    let found = sequence::load(&tiny_sequence(&base)).unwrap();
    assert_eq!(found.steps.iter().map(|step| step.id.as_str()).collect::<Vec<_>>(), ["01-hola", "02-nombre"]);
    let engine = fake_engine();
    let plan = Plan {
        sequence: &found,
        condition: Condition::C0,
        rep: 1,
        model: "claude-sonnet-5-5".into(),
        effort: "medium".into(),
        scratch: base.join("scratch"),
        out: base.join("out"),
        jscpd: root().join("../../node_modules/jscpd/run-jscpd.js"),
        patience: Duration::from_secs(60),
    };

    let first = sequence::step(&engine, &plan, 0, None).unwrap();
    let work = base.join("scratch").join(plan.name());
    std::fs::write(work.join("src").join("left.ts"), "export const half = 1;\n").unwrap();
    let second = sequence::step(&engine, &plan, 1, Some(&first.commit)).unwrap();
    engine.shutdown();

    assert!(first.error.is_empty() && first.accepted && first.finished, "{first:?}");
    assert!(second.accepted && second.broken.is_empty(), "{second:?}");
    assert!(!work.join("src").join("left.ts").exists());
    assert_ne!(first.commit, second.commit);
    assert_eq!((second.step, second.task.as_str()), (2, "02-nombre"));
    assert_eq!((second.shape.lines, second.shape.files, second.shape.duplicated), (3, 1, 0));
    assert_eq!(second.shape.probes["saludos"], 1);
    assert!(base.join("out").join("diffs").join("saludos-C0-1-02.diff").is_file());
    sequence::append(&base.join("out"), &first).unwrap();
    sequence::append(&base.join("out"), &second).unwrap();
    let back = sequence::read(&base.join("out")).unwrap();
    assert_eq!(sequence::last_commits(&back)[&(Condition::C0, 1)], (2, second.commit.clone()));
}
