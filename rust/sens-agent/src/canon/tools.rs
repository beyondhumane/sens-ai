use std::collections::BTreeSet;

use sens_canon::card;
use sens_canon::design::Guide;
use sens_canon::hot::{Heat, Reach};
use sens_index::format;
use sens_index::query::Engine;
use serde_json::{Value, json};

use super::keeper::Project;
use super::view::{self, Reached};

pub const NAMES: [&str; 11] = ["project_map", "file_outline", "find_symbol", "who_uses", "already_exists", "dead_code", "where_is", "impact", "tests_for", "design", "design_search"];
const SHOWN: usize = 25;
const NO_DESIGN: &str = "This project has no `Design/` folder yet, so it has no game design for Sens to follow. It starts with `Design/README.md` (the pitch and `subgenres:`), `Design/pillars.md` (each pillar a `##` heading with a `### Descarta` section), `Design/loop.md` and one file per mechanic in `Design/mechanics/`, with `pillars:` and `status:` in its frontmatter.";

fn described(name: &str, title: &str, description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "name": name,
        "title": title,
        "description": description,
        "inputSchema": { "type": "object", "properties": properties, "required": required, "additionalProperties": false },
        "annotations": { "readOnlyHint": true, "openWorldHint": false }
    })
}

pub fn listed() -> Vec<Value> {
    let folder = json!({ "folder": { "type": "string", "description": "Only this folder of the project, relative to its root. The whole project if left out." } });
    vec![
        described(
            "project_map",
            "Project map",
            "Without a folder: the project as it is now, by areas, with the file each area is entered through, the areas it uses, the central files and the files out of place. With a folder: its files with their symbols, one line each. Start here instead of reading files.",
            folder.clone(),
            &[],
        ),
        described("file_outline", "File outline", "The symbols of one file with their lines and signatures, without reading the file. In a Unity project it marks the methods that run every frame and how many calls away from it they are.", json!({ "file": { "type": "string", "description": "Path relative to the project root." } }), &["file"]),
        described("find_symbol", "Find symbol", "Where a function, type, class or constant is defined, by its exact name.", json!({ "name": { "type": "string" } }), &["name"]),
        described("who_uses", "Who uses", "Every place that uses a symbol, grouped by file.", json!({ "name": { "type": "string" } }), &["name"]),
        described(
            "already_exists",
            "Already exists",
            "Code in the project that already does what you describe, ranked by what it does, not only by its name. Ask before writing any new function, helper or component; the words can be in any language.",
            json!({ "query": { "type": "string", "description": "What the code should do, in a few words." } }),
            &["query"],
        ),
        described("dead_code", "Dead code", "Code no entry point reaches, ranked by how sure Sens is that it is unused.", folder, &[]),
        described(
            "where_is",
            "Where is",
            "Where a feature or concept lives, grouped by area, with the file each area is entered through. The words can be in any language.",
            json!({ "query": { "type": "string", "description": "The feature or concept, in a few words." } }),
            &["query"],
        ),
        described(
            "impact",
            "Impact",
            "What a change to a file or symbol can reach: the files that depend on it, step by step up to three steps, the areas they are in and the tests that reach it. Ask before changing something other code uses.",
            json!({ "target": { "type": "string", "description": "A path relative to the project root, or the exact name of a symbol." } }),
            &["target"],
        ),
        described(
            "tests_for",
            "Tests for",
            "The tests that reach a file, nearest first, so you run those instead of the whole suite.",
            json!({ "file": { "type": "string", "description": "Path relative to the project root." } }),
            &["file"],
        ),
        described(
            "design",
            "Game design",
            "The game's design from Design/: each pillar with its id and whether it says what it rules out, the core loop, every mechanic with its status and the pillars it serves, and the problems Sens found. Read it before you add or change gameplay.",
            json!({}),
            &[],
        ),
        described(
            "design_search",
            "Search the design",
            "The parts of the game's design (Design/) that answer a question: a mechanic, a rule, a number. The words can be in any language.",
            json!({ "query": { "type": "string", "description": "What you want to know about the design, in a few words." } }),
            &["query"],
        ),
    ]
}

fn design_search(project: &Project, query: &str) -> String {
    match &project.guide {
        None => NO_DESIGN.to_string(),
        Some(guide) => guide.relevant(query).map_or_else(|| "nothing in Design/ matches".to_string(), |told| told.text),
    }
}

fn said(arguments: &Value, key: &str) -> Result<String, String> {
    arguments[key].as_str().map(str::trim).filter(|text| !text.is_empty()).map(str::to_string).ok_or_else(|| format!("`{key}` is missing"))
}

fn existing(project: &Project, query: &str) -> String {
    let found = project.catalog.relevant(query);
    if found.is_empty() {
        return "nothing in the project matches".into();
    }
    found
        .iter()
        .map(|suggestion| {
            let symbol = &project.index.symbols[suggestion.symbol];
            format!("{}:{}  {}  · used {} times", symbol.file, symbol.line, symbol.signature.trim(), suggestion.uses)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn plural(count: usize, one: &str) -> String {
    if count == 1 { format!("1 {one}") } else { format!("{count} {one}s") }
}

fn every_frame(reach: Reach) -> Option<String> {
    match reach {
        Reach::Frame => Some("runs every frame".into()),
        Reach::Calls { count, from } => Some(format!("every frame, {} from {from}", plural(count as usize, "call"))),
        Reach::Far { from } => Some(format!("maybe every frame: {from} reaches it through more than 3 calls or a call Sens cannot tell apart")),
        Reach::Cold => None,
    }
}

fn outline(project: &Project, engine: &Engine, file: &str) -> String {
    let symbols = engine.file_outline(file);
    let told = format::format_symbols(&symbols);
    if project.index.unity.is_empty() || symbols.is_empty() {
        return told;
    }
    let heat = Heat::of(&project.index, None);
    told.lines()
        .zip(&symbols)
        .map(|(line, symbol)| every_frame(heat.reach(&symbol.name)).map_or_else(|| line.to_string(), |said| format!("{line}  [{said}]")))
        .collect::<Vec<_>>()
        .join("\n")
}

fn where_is(project: &Project, query: &str) -> String {
    let (index, map) = (&project.index, &project.map);
    let mut grouped: Vec<(usize, Vec<String>)> = Vec::new();
    for suggestion in project.catalog.relevant(query) {
        let symbol = &index.symbols[suggestion.symbol];
        let Some(file) = map.slot(&symbol.file) else {
            continue;
        };
        let line = format!("  {}:{}  {}", symbol.file, symbol.line, symbol.signature.trim());
        match grouped.iter_mut().find(|(area, _)| *area == map.area_of[file]) {
            Some((_, lines)) => lines.push(line),
            None => grouped.push((map.area_of[file], vec![line])),
        }
    }
    if grouped.is_empty() {
        return "nothing in the project matches".into();
    }
    grouped
        .into_iter()
        .map(|(at, lines)| {
            let area = &map.areas[at];
            let doors: Vec<&str> = area.doors.iter().map(|&door| index.files[door].path.as_str()).collect();
            let entered = if doors.is_empty() { String::new() } else { format!(" — entered through {}", doors.join(", ")) };
            format!("{}{entered}\n{}", area.name, lines.join("\n"))
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn reached(project: &Project, engine: &Engine, target: &str) -> Result<(String, Vec<Reached>, Vec<Reached>), String> {
    if let Some(reach) = view::reach(project, target) {
        return Ok((reach.file, reach.dependents, reach.tests));
    }
    let used = engine.who_uses(target);
    if used.is_empty() {
        return Err(format!("`{target}` is neither a file nor a symbol of this project"));
    }
    let map = &project.map;
    let defined: BTreeSet<&str> = used.iter().map(|found| found.symbol.file.as_str()).collect();
    let users: BTreeSet<usize> = used.iter().flat_map(|found| &found.references).filter(|site| !defined.contains(site.file.as_str())).filter_map(|site| map.slot(&site.file)).collect();
    let distances = map
        .spread(&users.into_iter().collect::<Vec<_>>(), usize::MAX)
        .into_iter()
        .filter(|&(file, _)| !defined.contains(project.index.files[file].path.as_str()))
        .map(|(file, distance)| (file, distance + 1));
    let (dependents, tests) = view::sorted(project, distances);
    Ok((format!("`{target}` ({})", defined.into_iter().collect::<Vec<_>>().join(", ")), dependents, tests))
}

fn impact(project: &Project, engine: &Engine, target: &str) -> Result<String, String> {
    let (origin, dependents, tests) = reached(project, engine, target)?;
    if dependents.is_empty() && tests.is_empty() {
        return Ok(format!("Nothing in the project depends on {origin}."));
    }
    let mut lines = vec![format!("What depends on {origin}, up to {} steps:", view::REACH)];
    for step in 1..=view::REACH {
        let at: Vec<&Reached> = dependents.iter().filter(|found| found.steps == step).collect();
        if at.is_empty() {
            continue;
        }
        lines.push(format!("{} — {}", plural(step, "step"), plural(at.len(), "file")));
        lines.extend(at.iter().take(SHOWN).map(|found| format!("  {}  [{}]", found.path, found.area)));
        if at.len() > SHOWN {
            lines.push(format!("  … and {} more", at.len() - SHOWN));
        }
    }
    let areas: BTreeSet<&str> = dependents.iter().map(|found| found.area.as_str()).collect();
    if !areas.is_empty() {
        lines.push(format!("Areas reached: {}", areas.into_iter().collect::<Vec<_>>().join(", ")));
    }
    if !tests.is_empty() {
        lines.push(format!("Tests that reach it: {}", tests.iter().map(|found| found.path.as_str()).collect::<Vec<_>>().join(", ")));
    }
    Ok(lines.join("\n"))
}

fn tests_for(project: &Project, file: &str) -> Result<String, String> {
    let reach = view::reach(project, file).ok_or_else(|| format!("`{file}` is not a file of this project"))?;
    let mut lines: Vec<String> = reach.tests.iter().map(|found| format!("{}  ({})", found.path, plural(found.steps, "step"))).collect();
    if project.index.units.iter().any(|unit| unit.test && unit.file == reach.file) {
        lines.insert(0, format!("{} carries its own tests", reach.file));
    }
    if lines.is_empty() {
        return Ok(format!("No test reaches {}.", reach.file));
    }
    Ok(lines.join("\n"))
}

pub fn call(project: &Project, name: &str, arguments: &Value) -> Option<Result<String, String>> {
    let engine = Engine::new(&project.index, &project.index.entry_points);
    let folder = arguments["folder"].as_str().filter(|folder| !folder.trim().is_empty());
    Some(match name {
        "project_map" => Ok(match folder {
            Some(folder) => format::format_map(&engine.map(Some(folder))),
            None => card::card(&project.index, &project.map),
        }),
        "file_outline" => said(arguments, "file").map(|file| outline(project, &engine, &file)),
        "find_symbol" => said(arguments, "name").map(|name| format::format_symbols(&engine.find_symbol(&name))),
        "who_uses" => said(arguments, "name").map(|name| format::format_who_uses(&engine.who_uses(&name), false)),
        "already_exists" => said(arguments, "query").map(|query| existing(project, &query)),
        "dead_code" => Ok(format::format_dead_code(&engine.dead_code(folder))),
        "where_is" => said(arguments, "query").map(|query| where_is(project, &query)),
        "impact" => said(arguments, "target").and_then(|target| impact(project, &engine, &target)),
        "tests_for" => said(arguments, "file").and_then(|file| tests_for(project, &file)),
        "design" => Ok(project.guide.as_ref().map_or_else(|| NO_DESIGN.to_string(), Guide::overview)),
        "design_search" => said(arguments, "query").map(|query| design_search(project, &query)),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::fixture::written;

    const CODE: [(&str, &str); 7] = [
        ("package.json", r#"{ "main": "src/index.ts" }"#),
        ("src/index.ts", "import { weigh } from './format.ts';\nweigh(10);\n"),
        ("src/format.ts", "export const weigh = (bytes: number) => `${Math.round(bytes / 1024)} KB`;\nexport function forgotten() { return 1; }\n"),
        ("src/view/size.ts", "import { weigh } from '../format.ts';\nexport const size = (bytes: number) => weigh(bytes);\n"),
        ("src/view/badge.ts", "import { size } from './size.ts';\nexport const badge = () => size(1);\n"),
        ("src/view/panel.ts", "import { badge } from './badge.ts';\nexport const panel = () => badge();\n"),
        ("test/badge.test.ts", "import { badge } from '../src/view/badge.ts';\nbadge();\n"),
    ];

    fn project_with(name: &str, more: &[(&str, &str)]) -> Project {
        let files: Vec<(&str, &str)> = CODE.iter().chain(more).copied().collect();
        Project::of(&written("sens-canon-tools", name, &files))
    }

    fn project(name: &str) -> Project {
        project_with(name, &[])
    }

    #[test]
    fn the_outline_of_a_unity_script_marks_what_runs_every_frame() {
        let player = "using UnityEngine;

public class Player : MonoBehaviour
{
    void Update() { Steer(); }
    void Steer() { }
    void Start() { }
}
";
        let project = project_with("unity-outline", &[("Assets/Scripts/Player.cs", player)]);
        let told = call(&project, "file_outline", &json!({ "file": "Assets/Scripts/Player.cs" })).unwrap().unwrap();
        let line = |name: &str| told.lines().find(|line| line.contains(name)).unwrap_or_else(|| panic!("{name} missing in:\n{told}"));
        assert!(line("void Update()").ends_with("[runs every frame]"), "{told}");
        assert!(line("void Steer()").ends_with("[every frame, 1 call from Player.Update]"), "{told}");
        assert!(!line("void Start()").contains('['), "{told}");
        assert!(!call(&project, "file_outline", &json!({ "file": "src/format.ts" })).unwrap().unwrap().contains("every frame"));
    }

    #[test]
    fn every_listed_tool_answers_and_only_those() {
        let project = project("all");
        assert_eq!(listed().iter().map(|tool| tool["name"].as_str().unwrap()).collect::<Vec<_>>(), NAMES);
        let answer = |name: &str, arguments: Value| call(&project, name, &arguments).unwrap();
        assert!(answer("project_map", json!({})).unwrap().contains("## This project"));
        assert!(answer("project_map", json!({ "folder": "src" })).unwrap().contains("src/format.ts"));
        assert!(answer("file_outline", json!({ "file": "src/format.ts" })).unwrap().contains("weigh"));
        assert!(answer("find_symbol", json!({ "name": "weigh" })).unwrap().contains("src/format.ts:1"));
        assert!(answer("who_uses", json!({ "name": "weigh" })).unwrap().contains("src/index.ts"));
        assert!(answer("already_exists", json!({ "query": "tamaño en bytes" })).unwrap().contains("weigh"));
        assert!(answer("dead_code", json!({})).unwrap().contains("forgotten"));
        assert!(answer("where_is", json!({ "query": "tamaño en bytes" })).unwrap().contains("src/format.ts:1"));
        assert!(answer("find_symbol", json!({})).unwrap_err().contains("name"));
        assert!(call(&project, "rm", &json!({})).is_none());
    }

    #[test]
    fn the_design_is_told_whole_or_searched_and_a_project_without_one_is_told_how_to_start() {
        let designed = project_with(
            "design",
            &[
                ("Design/README.md", "---\nsubgenres: [platformer]\n---\n# Pájaro\n"),
                ("Design/pillars.md", "## Movimiento preciso\n\nEl jugador siempre sabe por qué falló un salto.\n\n### Descarta\n\nAleatoriedad.\n\n## Riesgo\n\nArriesgar paga.\n"),
                ("Design/loop.md", "# Bucle\n\n## Segundos\n\nSaltar, agarrarse, impulsarse.\n"),
                ("Design/mechanics/dash.md", "---\npillars: [movimiento-preciso]\nstatus: prototype\n---\n# Dash\n\nUn impulso corto en el aire.\n\n## Números\n\nEl dash dura 0,15 s.\n"),
                ("Design/mechanics/hook.md", "# Gancho\n\nSe engancha a los anillos.\n"),
            ],
        );
        let answer = |name: &str, arguments: Value| call(&designed, name, &arguments).unwrap();
        let whole = answer("design", json!({})).unwrap();
        for line in [
            "- Movimiento preciso [movimiento-preciso] — El jugador siempre sabe por qué falló un salto.\n",
            "- Riesgo [riesgo] — Arriesgar paga. (rules nothing out)",
            "Subgenres: platformer.",
            "- Bucle › Segundos: Saltar, agarrarse, impulsarse.",
            "- Dash — prototype — serves movimiento-preciso — Design/mechanics/dash.md",
            "- Gancho — no status — serves no pillar — Design/mechanics/hook.md",
            "Problems Sens found in Design/:\n- ",
        ] {
            assert!(whole.contains(line), "{line}\n---\n{whole}");
        }
        let found = answer("design_search", json!({ "query": "cuánto dura el dash" })).unwrap();
        assert!(found.contains("### Dash › Números — Design/mechanics/dash.md") && found.contains("El dash dura 0,15 s."), "{found}");
        assert_eq!(answer("design_search", json!({ "query": "hola" })).unwrap(), "nothing in Design/ matches");
        assert!(answer("design_search", json!({})).unwrap_err().contains("query"));
        let bare = project("bare");
        assert_eq!(call(&bare, "design", &json!({})).unwrap().unwrap(), NO_DESIGN);
        assert_eq!(call(&bare, "design_search", &json!({ "query": "dash" })).unwrap().unwrap(), NO_DESIGN);
    }

    #[test]
    fn the_impact_of_a_file_or_a_symbol_goes_step_by_step_and_names_its_tests() {
        let project = project("impact");
        let answer = |arguments: Value| call(&project, "impact", &arguments).unwrap();
        let file = answer(json!({ "target": "src/format.ts" })).unwrap();
        assert!(file.contains("1 step — 2 files") && file.contains("src/view/size.ts  [src]"), "{file}");
        assert!(file.contains("3 steps — 1 file") && file.contains("src/view/panel.ts"), "{file}");
        assert!(file.contains("Tests that reach it: test/badge.test.ts"), "{file}");
        let symbol = answer(json!({ "target": "size" })).unwrap();
        assert!(symbol.starts_with("What depends on `size` (src/view/size.ts)") && symbol.contains("1 step — 1 file\n  src/view/badge.ts"), "{symbol}");
        assert!(answer(json!({ "target": "src/view/panel.ts" })).unwrap().starts_with("Nothing in the project depends on"));
        assert!(answer(json!({ "target": "nada" })).unwrap_err().contains("neither"));
    }

    #[test]
    fn the_tests_of_a_file_come_nearest_first() {
        let project = project("tests");
        let answer = |file: &str| call(&project, "tests_for", &json!({ "file": file })).unwrap();
        assert_eq!(answer("src/format.ts").unwrap(), "test/badge.test.ts  (3 steps)");
        assert_eq!(answer("view/badge.ts").unwrap(), "test/badge.test.ts  (1 step)");
        assert_eq!(answer("src/view/panel.ts").unwrap(), "No test reaches src/view/panel.ts.");
        assert!(answer("nowhere.ts").unwrap_err().contains("not a file"));
    }
}
