use std::collections::BTreeMap;

use sens_index::index::Index;
use sens_index::testfile::is_test_file;

use crate::dependencies;

const MAX_LINES: usize = 60;
const MODULES: usize = 20;
const EXPORTS: usize = 5;
const ENTRIES: usize = 8;
const NAMES: usize = 25;
const FOLDER_DEPTH: usize = 3;
const MANIFEST_DEPTH: usize = 4;
const SKIP: [&str; 7] = ["node_modules", "dist", "target", ".git", ".sens", "__pycache__", ".venv"];

type Module = (usize, Vec<(usize, String)>);

fn folder(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let depth = (parts.len() - 1).min(FOLDER_DEPTH);
    match depth {
        0 => ".".into(),
        _ => parts[..depth].join("/"),
    }
}

fn capped(names: &[String], cap: usize) -> String {
    let shown = names.iter().take(cap).cloned().collect::<Vec<_>>().join(", ");
    if names.len() > cap { format!("{shown}, …") } else { shown }
}

pub fn manifests(index: &Index) -> Vec<(String, Vec<String>)> {
    let mut manifests: Vec<(String, Vec<String>)> = ignore::WalkBuilder::new(&index.root)
        .hidden(false)
        .require_git(false)
        .max_depth(Some(MANIFEST_DEPTH))
        .filter_entry(|entry| !SKIP.contains(&entry.file_name().to_str().unwrap_or("")))
        .build()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let relative = entry.path().strip_prefix(&index.root).ok()?.to_string_lossy().replace('\\', "/");
            dependencies::is_manifest(&relative).then_some((relative, entry.into_path()))
        })
        .filter_map(|(relative, path)| {
            let names: Vec<String> = dependencies::declared(&relative, &std::fs::read_to_string(path).ok()?).into_iter().collect();
            (!names.is_empty()).then_some((relative, names))
        })
        .collect();
    manifests.sort();
    manifests
}

fn installed(index: &Index) -> Vec<String> {
    manifests(index).into_iter().map(|(manifest, names)| format!("- {manifest}: {}", capped(&names, NAMES))).collect()
}

pub fn card(index: &Index) -> String {
    let mut languages: BTreeMap<&str, usize> = BTreeMap::new();
    let mut modules: BTreeMap<String, Module> = BTreeMap::new();
    for file in index.files.iter().filter(|file| !is_test_file(&file.path)) {
        *languages.entry(file.language).or_default() += 1;
        modules.entry(folder(&file.path)).or_default().0 += 1;
    }
    for (at, symbol) in index.symbols.iter().enumerate().filter(|(_, symbol)| symbol.exported && symbol.kind != "method" && !is_test_file(&symbol.file)) {
        if let Some((_, exports)) = modules.get_mut(&folder(&symbol.file)) {
            exports.push((index.raw_references(at).len(), symbol.name.clone()));
        }
    }

    let mut lines = vec!["## This project, as Sens indexed it".to_string(), String::new()];
    let mut spoken: Vec<(&str, usize)> = languages.into_iter().collect();
    spoken.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    lines.push(format!("Languages: {}", spoken.iter().map(|(language, count)| format!("{language} ({count} files)")).collect::<Vec<_>>().join(", ")));
    let mut entries: Vec<String> = index.entry_points.iter().filter(|path| !is_test_file(path)).cloned().collect();
    entries.extend(index.symbols.iter().filter(|symbol| symbol.entry && symbol.name == "main" && !is_test_file(&symbol.file)).map(|symbol| symbol.file.clone()));
    entries.sort();
    entries.dedup();
    if !entries.is_empty() {
        lines.push(format!("Entry points: {}", capped(&entries, ENTRIES)));
    }
    let mut ranked: Vec<(String, Module)> = modules.into_iter().collect();
    ranked.sort_by(|a, b| b.1.0.cmp(&a.1.0).then(a.0.cmp(&b.0)));
    lines.push(String::new());
    lines.push("Modules, with their most used exports:".into());
    for (name, (count, mut exports)) in ranked.into_iter().take(MODULES) {
        exports.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut shown: Vec<String> = Vec::new();
        for (_, name) in exports {
            if !shown.contains(&name) {
                shown.push(name);
            }
        }
        let tail = if shown.is_empty() { String::new() } else { format!(": {}", capped(&shown, EXPORTS)) };
        lines.push(format!("- {name} ({count} files){tail}"));
    }
    let manifests = installed(index);
    if !manifests.is_empty() {
        lines.push(String::new());
        lines.push("Installed dependencies, to use before adding any:".into());
        lines.extend(manifests);
    }
    lines.truncate(MAX_LINES);
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::build;

    #[test]
    fn the_card_names_languages_entries_modules_and_what_is_installed() {
        let root = std::env::temp_dir().join("sens-canon-card");
        let _ = std::fs::remove_dir_all(&root);
        let files = [
            ("package.json", r#"{ "main": "src/index.ts", "dependencies": { "dayjs": "1" }, "devDependencies": { "prettier": "3" } }"#),
            ("src/index.ts", "import { formatBytes } from './lib/format.ts';\nformatBytes(1);\nformatBytes(2);\n"),
            ("src/lib/format.ts", "export function formatBytes(bytes: number) { return `${bytes} B`; }\nexport function unused() { return 1; }\n"),
            ("src/lib/time.ts", "export function ago(date: Date) { return date; }\n"),
            ("test/format.test.ts", "export function helper() {}\n"),
        ];
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        let text = card(&build::build(&root));
        assert!(text.contains("Languages: typescript (3 files)"), "{text}");
        assert!(text.contains("Entry points: src/index.ts"), "{text}");
        assert!(text.contains("- src/lib (2 files): formatBytes, ago, unused"), "{text}");
        assert!(text.contains("- package.json: dayjs, prettier"), "{text}");
        assert!(!text.contains("helper"), "{text}");
        assert!(text.lines().count() <= MAX_LINES);
    }

    #[test]
    fn a_folder_is_cut_to_three_levels() {
        assert_eq!(folder("main.rs"), ".");
        assert_eq!(folder("src/lib/format.ts"), "src/lib");
        assert_eq!(folder("rust/sens-app/ui/src/features/chat/Step.tsx"), "rust/sens-app/ui");
    }
}
