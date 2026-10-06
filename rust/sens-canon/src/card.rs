use std::collections::BTreeMap;

use sens_index::index::Index;
use sens_index::map::Map;
use sens_index::testfile::is_test_file;

use crate::dependencies;

const MAX_LINES: usize = 80;
const AREAS: usize = 25;
const EXPORTS: usize = 5;
const USES: usize = 4;
const ENTRIES: usize = 8;
const NAMES: usize = 25;
const STRAYS: usize = 6;
const MANIFEST_DEPTH: usize = 4;
const SKIP: [&str; 7] = ["node_modules", "dist", "target", ".git", ".sens", "__pycache__", ".venv"];

fn capped(names: &[String], cap: usize) -> String {
    let shown = names.iter().take(cap).cloned().collect::<Vec<_>>().join(", ");
    if names.len() > cap { format!("{shown}, …") } else { shown }
}

fn inside<'a>(path: &'a str, area: &str) -> &'a str {
    path.strip_prefix(area).and_then(|rest| rest.strip_prefix('/')).unwrap_or(path)
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

pub fn exports(index: &Index, map: &Map) -> Vec<Vec<String>> {
    let mut counted: Vec<Vec<(usize, &str)>> = vec![Vec::new(); map.areas.len()];
    for (at, symbol) in index.symbols.iter().enumerate().filter(|(_, symbol)| symbol.exported && symbol.kind != "method" && !is_test_file(&symbol.file)) {
        if let Some(file) = map.slot(&symbol.file) {
            counted[map.area_of[file]].push((index.raw_references(at).len(), symbol.name.as_str()));
        }
    }
    counted
        .into_iter()
        .map(|mut found| {
            found.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
            let mut names: Vec<String> = Vec::new();
            for (_, name) in found {
                if !names.iter().any(|known| known == name) {
                    names.push(name.to_string());
                }
            }
            names
        })
        .collect()
}

fn areas(index: &Index, map: &Map) -> Vec<String> {
    let exported = exports(index, map);
    let mut ranked: Vec<usize> = (0..map.areas.len()).filter(|&at| !map.areas[at].tests).collect();
    ranked.sort_by(|&a, &b| map.areas[b].files.len().cmp(&map.areas[a].files.len()).then(a.cmp(&b)));
    ranked
        .into_iter()
        .take(AREAS)
        .map(|at| {
            let area = &map.areas[at];
            let mut line = format!("- {} ({} files)", area.name, area.files.len());
            if !area.doors.is_empty() {
                let doors: Vec<String> = area.doors.iter().map(|&file| inside(&index.files[file].path, &area.name).to_string()).collect();
                line.push_str(&format!(" · door {}", doors.join(", ")));
            }
            if !area.uses.is_empty() {
                let used: Vec<String> = area.uses.iter().map(|&(used, _)| map.areas[used].name.clone()).collect();
                line.push_str(&format!(" · uses {}", capped(&used, USES)));
            }
            if !exported[at].is_empty() {
                line.push_str(&format!(": {}", capped(&exported[at], EXPORTS)));
            }
            line
        })
        .collect()
}

pub fn card(index: &Index, map: &Map) -> String {
    let mut languages: BTreeMap<&str, usize> = BTreeMap::new();
    for file in index.files.iter().filter(|file| !is_test_file(&file.path)) {
        *languages.entry(file.language).or_default() += 1;
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
    lines.push(String::new());
    lines.push("Areas, by folder: the file other areas enter through, the areas each one uses, and its most used exports:".into());
    lines.extend(areas(index, map));
    if !map.hubs.is_empty() {
        let hubs: Vec<String> = map.hubs.iter().map(|&(file, count)| format!("{} ({count})", index.files[file].path)).collect();
        lines.push(String::new());
        lines.push(format!("Central files, with how many files depend on each: {}", hubs.join(", ")));
    }
    if !map.strays.is_empty() {
        lines.push(String::new());
        lines.push("Files that work with another area more than with their own:".into());
        lines.extend(map.strays.iter().take(STRAYS).map(|&(file, area)| format!("- {} → {}", index.files[file].path, map.areas[area].name)));
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
    fn the_card_names_languages_entries_areas_and_what_is_installed() {
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
        let index = build::build(&root);
        let text = card(&index, &Map::of(&index));
        assert!(text.contains("Languages: typescript (3 files)"), "{text}");
        assert!(text.contains("Entry points: src/index.ts"), "{text}");
        assert!(text.contains("- src (3 files): formatBytes, ago, unused"), "{text}");
        assert!(text.contains("- package.json: dayjs, prettier"), "{text}");
        assert!(!text.contains("helper"), "{text}");
        assert!(text.lines().count() <= MAX_LINES);
    }
}
