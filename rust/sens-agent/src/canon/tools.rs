use sens_index::format;
use sens_index::query::Engine;
use serde_json::{Value, json};

use super::keeper::Project;

pub const NAMES: [&str; 6] = ["project_map", "file_outline", "find_symbol", "who_uses", "already_exists", "dead_code"];

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
        described("project_map", "Project map", "Files of the project with their symbols, one line each. Start here instead of reading files.", folder.clone(), &[]),
        described("file_outline", "File outline", "The symbols of one file with their lines and signatures, without reading the file.", json!({ "file": { "type": "string", "description": "Path relative to the project root." } }), &["file"]),
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
    ]
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

pub fn call(project: &Project, name: &str, arguments: &Value) -> Option<Result<String, String>> {
    let engine = Engine::new(&project.index, &project.index.entry_points);
    let folder = arguments["folder"].as_str().filter(|folder| !folder.trim().is_empty());
    Some(match name {
        "project_map" => Ok(format::format_map(&engine.map(folder))),
        "file_outline" => said(arguments, "file").map(|file| format::format_symbols(&engine.file_outline(&file))),
        "find_symbol" => said(arguments, "name").map(|name| format::format_symbols(&engine.find_symbol(&name))),
        "who_uses" => said(arguments, "name").map(|name| format::format_who_uses(&engine.who_uses(&name), false)),
        "already_exists" => said(arguments, "query").map(|query| existing(project, &query)),
        "dead_code" => Ok(format::format_dead_code(&engine.dead_code(folder))),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_canon::relevant::Catalog;

    fn project() -> Project {
        let root = std::env::temp_dir().join("sens-canon-tools");
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in [
            ("package.json", r#"{ "main": "src/index.ts" }"#),
            ("src/index.ts", "import { weigh } from './format.ts';\nweigh(10);\n"),
            ("src/format.ts", "export const weigh = (bytes: number) => `${Math.round(bytes / 1024)} KB`;\nexport function forgotten() { return 1; }\n"),
        ] {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        let index = sens_index::build::build(&root);
        let catalog = Catalog::of(&index);
        Project { index, catalog }
    }

    #[test]
    fn every_listed_tool_answers_and_only_those() {
        let project = project();
        assert_eq!(listed().iter().map(|tool| tool["name"].as_str().unwrap()).collect::<Vec<_>>(), NAMES);
        let answer = |name: &str, arguments: Value| call(&project, name, &arguments).unwrap();
        assert!(answer("project_map", json!({})).unwrap().contains("src/format.ts"));
        assert!(answer("file_outline", json!({ "file": "src/format.ts" })).unwrap().contains("weigh"));
        assert!(answer("find_symbol", json!({ "name": "weigh" })).unwrap().contains("src/format.ts:1"));
        assert!(answer("who_uses", json!({ "name": "weigh" })).unwrap().contains("src/index.ts"));
        assert!(answer("already_exists", json!({ "query": "tamaño en bytes" })).unwrap().contains("weigh"));
        assert!(answer("dead_code", json!({})).unwrap().contains("forgotten"));
        assert!(answer("find_symbol", json!({})).unwrap_err().contains("name"));
        assert!(call(&project, "rm", &json!({})).is_none());
    }
}
