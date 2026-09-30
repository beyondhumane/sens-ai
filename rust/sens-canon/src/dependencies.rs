use std::collections::BTreeSet;

use serde_json::Value;

use crate::verdict::{Change, Finding, Rule, Severity};

const NODE_SECTIONS: &[&str] = &["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"];
const CARGO_SECTIONS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];
const COMPOSER_SECTIONS: &[&str] = &["require", "require-dev"];
const GRADLE_CONFIGURATIONS: &[&str] = &[
    "implementation",
    "api",
    "compileOnly",
    "runtimeOnly",
    "testImplementation",
    "testRuntimeOnly",
    "testCompileOnly",
    "annotationProcessor",
    "kapt",
    "ksp",
    "debugImplementation",
    "releaseImplementation",
];

enum Manifest {
    Node,
    Cargo,
    Pyproject,
    Requirements,
    Go,
    Dotnet,
    Composer,
    Gemfile,
    Gradle,
    Maven,
}

fn manifest(path: &str) -> Option<Manifest> {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    match name {
        "package.json" => Some(Manifest::Node),
        "Cargo.toml" => Some(Manifest::Cargo),
        "pyproject.toml" => Some(Manifest::Pyproject),
        "go.mod" => Some(Manifest::Go),
        "composer.json" => Some(Manifest::Composer),
        "Gemfile" => Some(Manifest::Gemfile),
        "build.gradle" | "build.gradle.kts" => Some(Manifest::Gradle),
        "pom.xml" => Some(Manifest::Maven),
        _ if name.ends_with(".csproj") => Some(Manifest::Dotnet),
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
        Some(Manifest::Go) => go(content),
        Some(Manifest::Dotnet) => attributes(content, "<PackageReference", "Include=\""),
        Some(Manifest::Composer) => composer(content),
        Some(Manifest::Gemfile) => content.lines().filter_map(|line| line.trim().strip_prefix("gem ").and_then(quoted)).collect(),
        Some(Manifest::Gradle) => gradle(content),
        Some(Manifest::Maven) => maven(content),
        None => BTreeSet::new(),
    }
}

pub fn added(path: &str, before: &str, after: &str) -> Vec<String> {
    let known = declared(path, before);
    declared(path, after).into_iter().filter(|name| !known.contains(name)).collect()
}

pub fn findings(change: &Change) -> Vec<Finding> {
    if !is_manifest(&change.path) {
        return Vec::new();
    }
    added(&change.path, change.before(), change.after())
        .into_iter()
        .map(|name| Finding {
            rule: Rule::R3,
            severity: Severity::Ask,
            file: change.path.clone(),
            line: 1,
            message: format!("`{name}` would be a new dependency in {}. New dependencies need the person's approval; if the standard library or something already installed covers the need, use that instead.", change.path),
            target: None,
            key: format!("R3:{name}"),
        })
        .collect()
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

fn go(content: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut inside = false;
    for line in content.lines().map(|line| line.split("//").next().unwrap_or("").trim()) {
        if line.starts_with("require (") || line == "require(" {
            inside = true;
        } else if inside && line == ")" {
            inside = false;
        } else if let Some(single) = line.strip_prefix("require ") {
            found.extend(single.split_whitespace().next().map(str::to_string));
        } else if inside {
            found.extend(line.split_whitespace().next().map(str::to_string));
        }
    }
    found
}

fn quoted(text: &str) -> Option<String> {
    let start = text.find(['"', '\''])?;
    let quote = text[start..].chars().next()?;
    let rest = &text[start + 1..];
    let end = rest.find(quote)?;
    (end > 0).then(|| rest[..end].to_string())
}

fn attributes(content: &str, element: &str, attribute: &str) -> BTreeSet<String> {
    content
        .split(element)
        .skip(1)
        .filter_map(|tail| {
            let tag = tail.split('>').next()?;
            let value = tag.split(attribute).nth(1)?;
            value.split('"').next().filter(|name| !name.is_empty()).map(str::to_string)
        })
        .collect()
}

fn composer(content: &str) -> BTreeSet<String> {
    let Ok(parsed) = serde_json::from_str::<Value>(content) else {
        return BTreeSet::new();
    };
    COMPOSER_SECTIONS
        .iter()
        .filter_map(|section| parsed[section].as_object())
        .flat_map(|section| section.keys().cloned())
        .filter(|name| name != "php" && !name.starts_with("ext-"))
        .collect()
}

fn gradle(content: &str) -> BTreeSet<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| GRADLE_CONFIGURATIONS.iter().any(|configuration| line.strip_prefix(configuration).is_some_and(|rest| rest.starts_with(['(', ' ']))))
        .filter_map(quoted)
        .filter_map(|coordinates| {
            let mut parts = coordinates.split(':');
            Some(format!("{}:{}", parts.next()?, parts.next()?))
        })
        .collect()
}

fn element<'a>(block: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find("</")? + start;
    Some(block[start..end].trim())
}

fn maven(content: &str) -> BTreeSet<String> {
    content
        .split("<dependency>")
        .skip(1)
        .filter_map(|block| {
            let block = block.split("</dependency>").next()?;
            Some(format!("{}:{}", element(block, "groupId")?, element(block, "artifactId")?))
        })
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
        for path in ["go.mod", "web/App.csproj", "composer.json", "Gemfile", "build.gradle", "app/build.gradle.kts", "pom.xml"] {
            assert!(is_manifest(path), "{path}");
        }
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
    fn go_modules_count_single_and_grouped_requires() {
        let content = "module x\n\ngo 1.23\n\nrequire github.com/a/b v1.0.0\n\nrequire (\n\tgolang.org/x/text v0.1.0 // indirect\n\tgithub.com/c/d v2.0.0\n)\n";
        assert_eq!(names(declared("go.mod", content)), ["github.com/a/b", "github.com/c/d", "golang.org/x/text"]);
    }

    #[test]
    fn dotnet_composer_and_ruby_manifests_are_read() {
        let project = r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13" /><PackageReference Version="1" Include="Serilog" /></ItemGroup></Project>"#;
        assert_eq!(names(declared("App/App.csproj", project)), ["Newtonsoft.Json", "Serilog"]);
        let composer = r#"{ "require": { "php": ">=8.2", "ext-json": "*", "guzzlehttp/guzzle": "^7" }, "require-dev": { "phpunit/phpunit": "^11" } }"#;
        assert_eq!(names(declared("composer.json", composer)), ["guzzlehttp/guzzle", "phpunit/phpunit"]);
        let gemfile = "source 'https://rubygems.org'\ngem 'rails', '~> 8.0'\ngem \"pg\"\n";
        assert_eq!(names(declared("Gemfile", gemfile)), ["pg", "rails"]);
    }

    #[test]
    fn gradle_and_maven_name_group_and_artifact() {
        let gradle = "dependencies {\n    implementation(\"com.squareup.okhttp3:okhttp:4.12.0\")\n    testImplementation 'junit:junit:4.13.2'\n    implementationSomething(\"x:y:1\")\n}\n";
        assert_eq!(names(declared("app/build.gradle.kts", gradle)), ["com.squareup.okhttp3:okhttp", "junit:junit"]);
        let pom = "<project><dependencies><dependency>\n<groupId>org.slf4j</groupId>\n<artifactId>slf4j-api</artifactId>\n</dependency></dependencies></project>";
        assert_eq!(names(declared("pom.xml", pom)), ["org.slf4j:slf4j-api"]);
    }

    #[test]
    fn each_new_dependency_is_a_question_for_the_person() {
        let change = Change { path: "package.json".into(), before: Some(r#"{ "dependencies": { "dayjs": "1" } }"#.into()), after: Some(r#"{ "dependencies": { "dayjs": "1", "moment": "2" } }"#.into()) };
        let found = findings(&change);
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].rule, found[0].severity, found[0].key.as_str()), (Rule::R3, Severity::Ask, "R3:moment"));
        assert!(findings(&Change { path: "src/a.ts".into(), before: None, after: Some("x".into()) }).is_empty());
    }

    #[test]
    fn only_the_new_names_count_as_added() {
        let before = r#"{ "dependencies": { "dayjs": "1" } }"#;
        let after = r#"{ "dependencies": { "dayjs": "2", "date-fns": "4" } }"#;
        assert_eq!(added("package.json", before, after), ["date-fns"]);
        assert!(added("package.json", before, "not json").is_empty());
    }
}
