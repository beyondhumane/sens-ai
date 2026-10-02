use std::collections::HashSet;

use sens_index::index::Index;
use sens_index::query::{DeadCandidate, Engine, Tier};

use crate::verdict::{Finding, Rule, Severity};

pub type Dead = HashSet<(String, String)>;

fn with_dead<T>(index: &Index, read: impl FnOnce(&[DeadCandidate]) -> T) -> T {
    let engine = Engine::new(index, &index.entry_points);
    read(&engine.dead_code(None).candidates)
}

pub fn dead(index: &Index) -> Dead {
    with_dead(index, |candidates| candidates.iter().map(|candidate| (candidate.symbol.file.clone(), candidate.symbol.name.clone())).collect())
}

pub fn findings(index: &Index, before: &Dead) -> Vec<Finding> {
    with_dead(index, |candidates| {
        candidates
            .iter()
            .filter(|candidate| !before.contains(&(candidate.symbol.file.clone(), candidate.symbol.name.clone())))
            .map(|candidate| {
                let symbol = candidate.symbol;
                Finding {
                    rule: Rule::R4,
                    severity: if candidate.tier == Tier::Low { Severity::Note } else { Severity::Block },
                    file: symbol.file.clone(),
                    line: symbol.line,
                    message: format!(
                        "`{}` ({}:{}) has no use after this turn. Delete it, or call it where it was meant to be used.",
                        symbol.name, symbol.file, symbol.line
                    ),
                    target: None,
                    key: format!("R4:{}:{}", symbol.file, symbol.name),
                }
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::build;

    fn project(name: &str, files: &[(&str, &str)]) -> Index {
        let root = std::env::temp_dir().join("sens-canon-orphans").join(name);
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        build::build(&root)
    }

    #[test]
    fn what_the_turn_leaves_without_use_is_found_and_old_debt_is_not() {
        let entry = r#"{ "main": "src/index.ts" }"#;
        let before = project("before", &[("package.json", entry), ("src/index.ts", "import { run } from './run.ts';\nrun();\n"), ("src/run.ts", "function helper() { return 1; }\nfunction forgotten() { return 2; }\nexport function run() { return helper(); }\n")]);
        let dead_before = dead(&before);
        assert!(dead_before.contains(&("src/run.ts".to_string(), "forgotten".to_string())));
        let after = project("after", &[("package.json", entry), ("src/index.ts", "import { run } from './run.ts';\nrun();\n"), ("src/run.ts", "function helper() { return 1; }\nfunction forgotten() { return 2; }\nfunction fresh() { return 3; }\nexport function run() { return 1; }\n")]);
        let found = findings(&after, &dead_before);
        let names: Vec<(&str, Severity)> = found.iter().map(|finding| (finding.key.as_str(), finding.severity)).collect();
        assert_eq!(names, [("R4:src/run.ts:helper", Severity::Block), ("R4:src/run.ts:fresh", Severity::Block)]);
    }
}
