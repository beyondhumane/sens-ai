use std::collections::BTreeSet;
use std::path::{MAIN_SEPARATOR, Path};

use serde_json::Value;

use crate::index::Index;

const FOLDER_ENTRIES: [&str; 4] = ["index.ts", "index.tsx", "index.js", "index.jsx"];
const SOURCE: [&str; 8] = ["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"];
const BUILT_INTO: [&str; 10] = ["dist", "build", "lib", "out", "es", "esm", "cjs", "umd", "types", "typings"];
const PACKAGE_FIELDS: [&str; 6] = ["main", "module", "types", "typings", "bin", "exports"];
const SKIP_DIRS: [&str; 5] = ["node_modules", "dist", ".sens", ".git", "target"];

pub fn find(root: &Path, index: &Index) -> Vec<String> {
    let targets = package_targets(root);
    index
        .files
        .iter()
        .map(|file| file.path.as_str())
        .filter(|path| {
            FOLDER_ENTRIES.contains(&path.rsplit('/').next().unwrap_or(path))
                || targets.iter().any(|(prefix, tail)| path.strip_prefix(prefix.as_str()).is_some_and(|inside| ends_as(inside, tail)))
        })
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn ends_as(inside: &str, tail: &str) -> bool {
    let Some((stem, extension)) = inside.rsplit_once('.') else {
        return false;
    };
    SOURCE.contains(&extension) && (stem == tail || stem.ends_with(&format!("/{tail}")))
}

fn leaves(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => out.push(text.clone()),
        Value::Array(items) => items.iter().for_each(|item| leaves(item, out)),
        Value::Object(fields) => fields.values().for_each(|item| leaves(item, out)),
        _ => {}
    }
}

fn tail_of(target: &str) -> Option<String> {
    let mut parts: Vec<&str> = target.trim_start_matches("./").split('/').filter(|part| !part.is_empty() && *part != ".").collect();
    while parts.len() > 1 && BUILT_INTO.contains(&parts[0]) {
        parts.remove(0);
    }
    let joined = parts.join("/");
    let stem = joined.strip_suffix(".d.ts").unwrap_or(&joined);
    let stem = SOURCE.iter().find_map(|extension| stem.strip_suffix(&format!(".{extension}"))).unwrap_or(stem);
    (!stem.is_empty()).then(|| stem.to_string())
}

fn package_targets(root: &Path) -> Vec<(String, String)> {
    ignore::WalkBuilder::new(root)
        .hidden(false)
        .require_git(false)
        .filter_entry(|entry| !SKIP_DIRS.contains(&entry.file_name().to_str().unwrap_or("")))
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() == "package.json")
        .flat_map(|entry| {
            let folder = entry.path().parent().and_then(|parent| parent.strip_prefix(root).ok()).map(|relative| relative.to_string_lossy().replace(MAIN_SEPARATOR, "/")).unwrap_or_default();
            let prefix = if folder.is_empty() { String::new() } else { format!("{folder}/") };
            let parsed: Value = std::fs::read_to_string(entry.path()).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null);
            let mut written = Vec::new();
            PACKAGE_FIELDS.iter().for_each(|field| leaves(&parsed[field], &mut written));
            written.into_iter().filter_map(|target| tail_of(&target)).map(move |tail| (prefix.clone(), tail)).collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::FileInfo;

    fn file(path: &str) -> FileInfo {
        FileInfo { path: path.into(), language: "typescript", mtime_ms: 0.0, exports: Vec::new() }
    }

    #[test]
    fn a_build_path_in_package_json_points_back_to_its_source() {
        assert_eq!(tail_of("./dist/cli.js").as_deref(), Some("cli"));
        assert_eq!(tail_of("lib/esm/api/index.mjs").as_deref(), Some("api/index"));
        assert_eq!(tail_of("./types/index.d.ts").as_deref(), Some("index"));
        assert_eq!(tail_of("./"), None);
    }

    #[test]
    fn folder_indexes_and_package_targets_are_entries_and_nothing_else() {
        let root = std::env::temp_dir().join("sens-index-entries");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("packages/cli")).unwrap();
        std::fs::write(root.join("package.json"), r#"{ "main": "./dist/server.js", "exports": { ".": { "import": "./dist/api.mjs" } } }"#).unwrap();
        std::fs::write(root.join("packages/cli/package.json"), r#"{ "bin": { "tool": "./build/main.js" } }"#).unwrap();
        let files = ["src/server.ts", "src/api.ts", "src/helper.ts", "src/ui/index.tsx", "packages/cli/src/main.ts", "src/main.ts"].map(file).to_vec();
        let index = Index::assemble(root.clone(), files, Vec::new(), Vec::new(), Default::default());
        assert_eq!(find(&root, &index), ["packages/cli/src/main.ts", "src/api.ts", "src/server.ts", "src/ui/index.tsx"]);
    }
}
