use std::path::Path;

use sens_index::build;

fn fingerprint(fixture: &str) -> String {
    let index = build::build(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(fixture));
    let references: Vec<Vec<(u32, u32, u32)>> = (0..index.symbols.len()).map(|at| index.raw_references(at).to_vec()).collect();
    format!("{:?}{:?}{:?}{:?}", index.files, index.symbols, index.imports, references)
}

#[test]
fn the_same_project_always_gives_the_same_index() {
    for fixture in ["polyglot", "rustdead", "cppdead", "cdead", "rubydead", "javadead", "pythondead", "kotlindead", "godead", "barrel", "monorepo"] {
        let first = fingerprint(fixture);
        for _ in 0..4 {
            assert!(fingerprint(fixture) == first, "{fixture} changed between builds");
        }
    }
}
