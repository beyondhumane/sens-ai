use std::collections::HashSet;

use sens_index::build;
use sens_index::index::Index;

use super::FOLDER;
use crate::unity;
use crate::verdict::{Change, Finding, Rule, Severity};

const GAMEPLAY: [&str; 2] = ["MonoBehaviour", "ScriptableObject"];

fn touches_design(path: &str) -> bool {
    path.split('/').next().is_some_and(|top| top.eq_ignore_ascii_case(FOLDER))
}

fn in_play(path: &str) -> bool {
    path.starts_with("Assets/") && path.ends_with(".cs") && unity::ships(path)
}

fn classes(path: &str, source: &str) -> Vec<(String, u32)> {
    build::parse(path, source).map_or_else(Vec::new, |(_, emitted)| emitted.symbols.into_iter().filter(|symbol| symbol.kind == "class").map(|symbol| (symbol.name, symbol.line)).collect())
}

fn undescribed_class(change: &Change, name: String, line: u32, root: &str) -> Finding {
    Finding {
        rule: Rule::D1,
        severity: Severity::Ask,
        file: change.path.clone(),
        line,
        message: format!(
            "`{name}` ({}:{line}) is a new {root}, and this turn did not touch `{FOLDER}/`. If it is a mechanic of the game, describe it in `{FOLDER}/mechanics/` with the pillars it serves and its status, then finish again. If it is not, the person can say so and Sens will not ask about `{name}` again.",
            change.path
        ),
        target: None,
        key: format!("D1:{name}"),
    }
}

pub fn undescribed(index: &Index, changes: &[Change]) -> Vec<Finding> {
    if !index.root.join(FOLDER).is_dir() || changes.iter().any(|change| touches_design(&change.path)) {
        return Vec::new();
    }
    let mut found = Vec::new();
    for change in changes.iter().filter(|change| in_play(&change.path)) {
        let known: HashSet<String> = classes(&change.path, change.before()).into_iter().map(|(name, _)| name).collect();
        for (name, line) in classes(&change.path, change.after()) {
            let Some(&root) = index.unity.get(&name).filter(|root| GAMEPLAY.contains(root)) else { continue };
            if !known.contains(&name) {
                found.push(undescribed_class(change, name, line, root));
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::fixture::indexed;

    const DESIGN: (&str, &str) = ("Design/README.md", "# Pájaro\n");
    const DASH: &str = "using UnityEngine;\n\npublic class Dash : MonoBehaviour\n{\n    void Update() { }\n}\n";

    fn added(path: &str, after: &str) -> Change {
        Change { path: path.into(), before: None, after: Some(after.into()) }
    }

    fn asked(name: &str, files: &[(&str, &str)], changes: &[Change]) -> Vec<String> {
        undescribed(&indexed("sens-canon-undescribed", name, files), changes).into_iter().map(|finding| finding.key).collect()
    }

    #[test]
    fn a_new_component_outside_the_design_is_asked_about_once_by_name() {
        let found = undescribed(&indexed("sens-canon-undescribed", "asked", &[DESIGN, ("Assets/Scripts/Dash.cs", DASH)]), &[added("Assets/Scripts/Dash.cs", DASH)]);
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].rule, found[0].severity, found[0].key.as_str(), found[0].line), (Rule::D1, Severity::Ask, "D1:Dash", 3));
        assert!(found[0].message.starts_with("`Dash` (Assets/Scripts/Dash.cs:3) is a new MonoBehaviour"), "{}", found[0].message);
    }

    #[test]
    fn touching_the_design_in_the_same_turn_answers_the_question() {
        let design = Change { path: "Design/mechanics/dash.md".into(), before: None, after: Some("# Dash\n".into()) };
        assert!(asked("touched", &[DESIGN, ("Assets/Scripts/Dash.cs", DASH)], &[added("Assets/Scripts/Dash.cs", DASH), design]).is_empty());
    }

    #[test]
    fn editor_and_test_folders_and_a_class_that_was_already_there_are_left_alone() {
        let files = [DESIGN, ("Assets/Editor/DashEditor.cs", "public class Tool : MonoBehaviour { }\n"), ("Assets/Tests/Play.cs", "public class Probe : MonoBehaviour { }\n"), ("Assets/Scripts/Dash.cs", DASH)];
        let changes = [
            added("Assets/Editor/DashEditor.cs", files[1].1),
            added("Assets/Tests/Play.cs", files[2].1),
            Change { path: "Assets/Scripts/Dash.cs".into(), before: Some(DASH.into()), after: Some(DASH.replace("{ }", "{ Move(); }")) },
        ];
        assert!(asked("aside", &files, &changes).is_empty());
    }

    #[test]
    fn a_component_through_a_base_in_another_file_counts_and_a_plain_class_does_not() {
        let boss = "public class Boss : Dash { }\npublic class Damage { }\n";
        let files = [DESIGN, ("Assets/Scripts/Dash.cs", DASH), ("Assets/Scripts/Boss.cs", boss)];
        assert_eq!(asked("inherited", &files, &[added("Assets/Scripts/Boss.cs", boss)]), ["D1:Boss"]);
    }

    #[test]
    fn without_a_design_folder_nothing_is_asked() {
        assert!(asked("no-design", &[("Assets/Scripts/Dash.cs", DASH)], &[added("Assets/Scripts/Dash.cs", DASH)]).is_empty());
    }
}
