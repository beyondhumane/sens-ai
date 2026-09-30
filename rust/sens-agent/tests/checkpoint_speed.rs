use std::path::Path;
use std::time::Instant;

use sens_agent::canon::checkpoint::Checkpoints;

#[test]
#[ignore]
fn a_snapshot_of_this_repository_stays_within_budget() {
    let root = std::path::absolute(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
    let checkpoints = Checkpoints::open(&root).unwrap();
    let started = Instant::now();
    let first = checkpoints.snapshot("speed/first").unwrap();
    let cold = started.elapsed();
    let started = Instant::now();
    let second = checkpoints.snapshot("speed/second").unwrap();
    let warm = started.elapsed();
    eprintln!("first snapshot {} ms, second {} ms", cold.as_millis(), warm.as_millis());
    assert_eq!(first, second);
    assert!(warm.as_millis() < 300, "{} ms", warm.as_millis());
}
