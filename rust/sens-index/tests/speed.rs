use std::path::Path;
use std::time::Instant;

use sens_index::build;
use sens_index::query::Engine;

#[test]
#[ignore]
fn indexing_this_repository_stays_within_budget() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let started = Instant::now();
    let index = build::build(&root);
    let built = started.elapsed();
    let started = Instant::now();
    let engine = Engine::new(&index, &index.entry_points);
    let dead = engine.dead_code_report(None).candidates.len();
    let queried = started.elapsed();
    let sites: usize = (0..index.symbols.len()).map(|at| index.raw_references(at).len()).sum();
    eprintln!(
        "{} files · {} symbols · {} units · {} references · {} imports · {} entry files · {} dead candidates · build {} ms · engine and dead code {} ms",
        index.files.len(),
        index.symbols.len(),
        index.units.len(),
        sites,
        index.imports.len(),
        index.entry_points.len(),
        dead,
        built.as_millis(),
        queried.as_millis()
    );
    let probe = index.units.iter().find(|unit| unit.comparable()).unwrap().print.clone();
    let started = Instant::now();
    for _ in 0..1_000 {
        assert!(!index.similar(&probe).is_empty());
    }
    eprintln!("similar: {} µs per lookup", started.elapsed().as_micros() / 1_000);
    assert!(index.files.len() > 100);
    assert!(built.as_millis() < 5_000, "{} ms", built.as_millis());
}
