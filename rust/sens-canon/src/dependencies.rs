use std::collections::BTreeSet;

use serde_json::Value;

const NODE_SECTIONS: &[&str] = &["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"];
const CARGO_SECTIONS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];

enum Manifest {
    Node,
    Cargo,
    Pyproject,
    Requirements,
}

fn manifest(path: &str) -> Option<Manifest> {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    match name {
        "package.json" => Some(Manifest::Node),
        "Cargo.toml" => Some(Manifest::Cargo),
        "pyproject.toml" => Some(Manifest::Pyproject),
        _ if name.starts_with("requirements") && name.ends_with(".txt") => Some(Manifest::Requirements),
        _ => None,
    }
}

pub fn is_manifest(path: &str) -> bool {
    manifest(path).is_some()
}

pub fn declared(path: &str, content: &str) -> BTreeSet<String> {
    match manifest(path) {
        Some(Manifest::Node) => node(content),
        Some(Manifest::Cargo) => cargo(content),
        Some(Manifest::Pyproject) => pyproject(content),
        Some(Manifest::Requirements) => content.lines().filter_map(requirement).collect(),
        None => BTreeSet::new(),
    }
}

pub fn added(path: &str, before: &str, after: &str) -> Vec<String> {
    let known = declared(path, before);
    declared(path, after).into_iter().filter(|name| !known.contains(name)).collect()
}

fn node(content: &str) -> BTreeSet<String> {
    let Ok(parsed) = serde_json::from_str::<Value>(content) else {
        return BTreeSet::new();
    };
    NODE_SECTIONS
        .iter()
        .filter_map(|section| parsed[section].as_object())
        .flat_map(|section| section.keys().cloned())
        .collect()
}

fn cargo(content: &str) -> BTreeSet<String> {
    let Ok(parsed) = content.parse::<toml::Table>() else {
        return BTreeSet::new();
    };
    let targets = parsed
        .get("target")
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|targets| targets.values().filter_map(toml::Value::as_table));
    std::iter::once(&parsed)
        .chain(targets)
        .flat_map(|table| CARGO_SECTIONS.iter().filter_map(|section| table.get(*section)?.as_table()))
        .flat_map(|section| section.keys().cloned())
        .collect()
}

fn pyproject(content: &str) -> BTreeSet<String> {
    let Ok(parsed) = content.parse::<toml::Table>() else {
        return BTreeSet::new();
    };
    let listed = |value: Option<&toml::Value>| -> Vec<String> {
        value
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
            .filter_map(requirement)
            .collect()
    };
    let grouped = |value: Option<&toml::Value>| -> Vec<String> {
        value
            .and_then(toml::Value::as_table)
            .into_iter()
            .flat_map(|groups| groups.values())
            .flat_map(|group| listed(Some(group)))
            .collect()
    };
    let project = parsed.get("project");
    let poetry = parsed
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("dependencies"))
        .and_then(toml::Value::as_table)
        .into_iter()
        .flat_map(|table| table.keys())
        .filter(|name| name.as_str() != "python")
        .map(|name| python_name(name));
    listed(project.and_then(|project| project.get("dependencies")))
        .into_iter()
        .chain(grouped(project.and_then(|project| project.get("optional-dependencies"))))
        .chain(grouped(parsed.get("dependency-groups")))
        .chain(poetry)
        .collect()
}

fn requirement(line: &str) -> Option<String> {
    let line = line.split('#').next()?.trim();
    if line.is_empty() || line.starts_with('-') {
        return None;
    }
    let name: String = line.chars().take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')).collect();
    (!name.is_empty()).then(|| python_name(&name))
}

fn python_name(name: &str) -> String {
    name.to_ascii_lowercase().replace(['_', '.'], "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(found: BTreeSet<String>) -> Vec<String> {
        found.into_iter().collect()
    }

    #[test]
    fn only_known_manifests_are_read() {
        assert!(is_manifest("web/package.json"));
        assert!(is_manifest("crates\\core\\Cargo.toml"));
        assert!(is_manifest("requirements-dev.txt"));
        assert!(!is_manifest("package.jsonc"));
        assert!(!is_manifest("src/main.rs"));
    }

    #[test]
    fn a_node_package_lists_every_kind_of_dependency() {
        let content = r#"{ "name": "x", "dependencies": { "dayjs": "1" }, "devDependencies": { "vitest": "4" }, "peerDependencies": { "react": "19" }, "scripts": { "test": "vitest" } }"#;
        assert_eq!(names(declared("package.json", content)), ["dayjs", "react", "vitest"]);
    }

    #[test]
    fn a_cargo_manifest_includes_platform_specific_dependencies() {
        let content = "[package]\nname = \"x\"\n[dependencies]\nserde = \"1\"\n[dev-dependencies]\ntempfile = \"3\"\n[target.'cfg(windows)'.dependencies]\nwindows = \"0.61\"\n";
        assert_eq!(names(declared("Cargo.toml", content)), ["serde", "tempfile", "windows"]);
    }

    #[test]
    fn python_names_are_normalized_wherever_they_are_declared() {
        let content = "[project]\ndependencies = [\"Requests>=2\", \"typing_extensions ; python_version < '3.11'\"]\n[project.optional-dependencies]\ncli = [\"rich[jupyter]==13\"]\n[tool.poetry.dependencies]\npython = \"^3.11\"\n\"Zope.Interface\" = \"*\"\n";
        assert_eq!(names(declared("pyproject.toml", content)), ["requests", "rich", "typing-extensions", "zope-interface"]);
    }

    #[test]
    fn requirements_skip_comments_options_and_blank_lines() {
        let content = "# pinned\n-r base.txt\n\nDjango==5.1  # web\nnumpy\n--index-url https://x\n";
        assert_eq!(names(declared("requirements.txt", content)), ["django", "numpy"]);
    }

    #[test]
    fn only_the_new_names_count_as_added() {
        let before = r#"{ "dependencies": { "dayjs": "1" } }"#;
        let after = r#"{ "dependencies": { "dayjs": "2", "date-fns": "4" } }"#;
        assert_eq!(added("package.json", before, after), ["date-fns"]);
        assert!(added("package.json", before, "not json").is_empty());
    }
}
