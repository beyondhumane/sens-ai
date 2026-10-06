use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::{Desk, Level, MOST_CHARS, Said, Scope, Tool, opening, schema, text};
use crate::capabilities::{self, NewServer};
use crate::market::{self, Kind};

const KINDS: [&str; 3] = ["skill", "server", "plugin"];
const NEXT: &str = "It applies from each session's next message.";

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "list_capabilities",
        title: "List capabilities",
        level: Level::Read,
        description: "The skills, MCP servers and plugins installed in Sens, with whether each is on for this project.",
        input: || schema(json!({}), &[]),
        run: list_capabilities,
    },
    Tool {
        name: "read_skill",
        title: "Read skill",
        level: Level::Read,
        description: "The whole SKILL.md of an installed skill.",
        input: || schema(json!({ "name": { "type": "string" } }), &["name"]),
        run: read_skill,
    },
    Tool {
        name: "create_skill",
        title: "Create skill",
        level: Level::Change,
        description: "Write a new skill into Sens, on for this project: a name in kebab-case, the one-line description that decides when it is used, and its instructions in Markdown.",
        input: || {
            schema(
                json!({
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "body": { "type": "string", "description": "The instructions, without front matter." }
                }),
                &["name", "description", "body"],
            )
        },
        run: create_skill,
    },
    Tool {
        name: "set_capability",
        title: "Turn capability on or off",
        level: Level::Change,
        description: "Turn an installed skill, MCP server or plugin on or off for this project.",
        input: || schema(json!({ "kind": { "type": "string", "enum": KINDS }, "name": { "type": "string" }, "enabled": { "type": "boolean" } }), &["kind", "name", "enabled"]),
        run: set_capability,
    },
    Tool {
        name: "remove_capability",
        title: "Remove capability",
        level: Level::Change,
        description: "Uninstall a skill, MCP server or plugin from Sens, for every project.",
        input: || schema(json!({ "kind": { "type": "string", "enum": KINDS }, "name": { "type": "string" } }), &["kind", "name"]),
        run: remove_capability,
    },
    Tool {
        name: "add_server",
        title: "Add MCP server",
        level: Level::Change,
        description: "Add a local MCP server that Sens starts with a command, on for this project. Servers that need keys or tokens are added by the person in Capabilities.",
        input: || {
            schema(
                json!({
                    "name": { "type": "string" },
                    "command": { "type": "string" },
                    "args": { "type": "array", "items": { "type": "string" } }
                }),
                &["name", "command"],
            )
        },
        run: add_server,
    },
    Tool {
        name: "market_search",
        title: "Search the market",
        level: Level::Read,
        description: "Search Sens's market of plugins, skills and connectors by what they do.",
        input: || schema(json!({ "query": { "type": "string" } }), &["query"]),
        run: market_search,
    },
    Tool {
        name: "market_detail",
        title: "Market detail",
        level: Level::Read,
        description: "Everything about one listing of the market: what it brings, its files, its readme and the values it asks for.",
        input: || schema(json!({ "id": { "type": "string" } }), &["id"]),
        run: market_detail,
    },
    Tool {
        name: "market_install",
        title: "Install from the market",
        level: Level::Change,
        description: "Install a listing of the market into Sens, on for this project, with the values it asks for. One that needs a secret (a key, a token) is installed by the person in Capabilities.",
        input: || schema(json!({ "id": { "type": "string" }, "values": { "type": "object", "additionalProperties": { "type": "string" } } }), &["id"]),
        run: market_install,
    },
    Tool {
        name: "market_update",
        title: "Update from the market",
        level: Level::Change,
        description: "Bring an installed plugin, skill or connector up to its latest version in the market.",
        input: || schema(json!({ "id": { "type": "string" }, "name": { "type": "string", "description": "Its installed name." } }), &["id", "name"]),
        run: market_update,
    },
];

fn root(scope: &Scope) -> String {
    scope.root().to_string_lossy().into_owned()
}

fn changed(desk: &Desk, done: String) -> Result<Said, String> {
    let _ = desk.ui.act("refresh_capabilities", json!({}));
    Ok(format!("{done} {NEXT}").into())
}

fn on(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}

fn list_capabilities(desk: &Desk, scope: &Scope, _: &Value) -> Result<Said, String> {
    let all = capabilities::all(&desk.data()?, &root(scope));
    let mut lines = vec!["Skills:".to_string()];
    lines.extend(all.skills.iter().map(|skill| format!("  {} · {} · {}", skill.name, on(skill.enabled), skill.description)));
    lines.push("MCP servers:".into());
    lines.extend(all.servers.iter().map(|server| {
        let how = if server.url.is_empty() { format!("{} {}", server.command, server.args.join(" ")) } else { server.url.clone() };
        format!("  {} · {} · {}", server.name, on(server.enabled), how.trim())
    }));
    lines.push("Plugins:".into());
    lines.extend(all.plugins.iter().map(|plugin| format!("  {} {} · {} · {}", plugin.name, plugin.version, on(plugin.enabled), plugin.description)));
    Ok(lines.join("\n").into())
}

fn read_skill(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    capabilities::skill_text(&desk.data()?, &text(arguments, "name")?).map(Said::from)
}

fn create_skill(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let name = text(arguments, "name")?;
    capabilities::create_skill(&desk.data()?, &root(scope), &name, &text(arguments, "description")?, &text(arguments, "body")?)?;
    changed(desk, format!("The skill {name} is written and on for this project."))
}

fn set_capability(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let (kind, name) = (text(arguments, "kind")?, text(arguments, "name")?);
    let enabled = arguments["enabled"].as_bool().ok_or("`enabled` is missing")?;
    let (base, root) = (desk.data()?, root(scope));
    match kind.as_str() {
        "skill" => capabilities::set_skill(&base, &root, &name, enabled),
        "server" => capabilities::set_server(&base, &root, &name, enabled),
        "plugin" => capabilities::set_plugin(&base, &root, &name, enabled),
        other => Err(format!("{other} is not a skill, server or plugin")),
    }?;
    changed(desk, format!("The {kind} {name} is {} for this project.", on(enabled)))
}

fn remove_capability(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let (kind, name) = (text(arguments, "kind")?, text(arguments, "name")?);
    let base = desk.data()?;
    match kind.as_str() {
        "skill" => capabilities::remove_skill(&base, &name),
        "server" => capabilities::remove_server(&base, &name),
        "plugin" => capabilities::remove_plugin(&base, &name),
        other => Err(format!("{other} is not a skill, server or plugin")),
    }?;
    changed(desk, format!("The {kind} {name} is uninstalled."))
}

fn add_server(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let name = text(arguments, "name")?;
    let args = arguments["args"].as_array().map(|args| args.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default();
    let server = NewServer { name: name.clone(), command: text(arguments, "command")?, args, env: BTreeMap::new() };
    capabilities::add_server(&desk.data()?, &root(scope), &server)?;
    changed(desk, format!("The MCP server {name} is added and on for this project."))
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Plugin => "plugin",
        Kind::Skill => "skill",
        Kind::Connector => "connector",
    }
}

fn market_search(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let found = market::search(&desk.data()?, &text(arguments, "query")?)?;
    let lines: Vec<String> = found
        .iter()
        .take(20)
        .map(|listing| {
            let how = if listing.installable { "" } else { " · not installable here" };
            format!("{} · {} {} by {} · {}{how}", listing.id, kind_name(listing.kind), listing.title, listing.author, listing.description)
        })
        .collect();
    Ok(if lines.is_empty() { "Nothing in the market matches.".into() } else { lines.join("\n").into() })
}

fn market_detail(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let detail = market::detail(&desk.data()?, &text(arguments, "id")?)?;
    let mut shown = serde_json::to_value(&detail).map_err(|error| error.to_string())?;
    if let Some(readme) = shown["readme"].as_str() {
        shown["readme"] = json!(readme.chars().take(MOST_CHARS / 2).collect::<String>());
    }
    let text = serde_json::to_string_pretty(&shown).map_err(|error| error.to_string())?;
    Ok(opening(&text).into())
}

fn market_install(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let id = text(arguments, "id")?;
    let base = desk.data()?;
    let detail = market::detail(&base, &id)?;
    if let Some(secret) = detail.needs.iter().find(|need| need.secret && need.required) {
        return Err(format!("{} needs a secret ({}); the person installs it themselves in Capabilities", detail.listing.title, secret.name));
    }
    let values: BTreeMap<String, String> = arguments["values"]
        .as_object()
        .map(|values| values.iter().filter_map(|(key, value)| value.as_str().map(|value| (key.clone(), value.to_string()))).collect())
        .unwrap_or_default();
    if let Some(secret) = detail.needs.iter().find(|need| need.secret && values.contains_key(&need.name)) {
        return Err(format!("{} is a secret; the person enters it themselves in Capabilities", secret.name));
    }
    let name = market::install(&base, &root(scope), &id, &values)?;
    changed(desk, format!("{} is installed as {name} and on for this project.", detail.listing.title))
}

fn market_update(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let name = text(arguments, "name")?;
    market::update(&desk.data()?, &text(arguments, "id")?, &name)?;
    changed(desk, format!("{name} is up to date."))
}

#[cfg(test)]
mod tests {
    use super::super::testing::{Window, scope};
    use super::*;
    use std::path::PathBuf;

    fn window(name: &str) -> (Window, PathBuf) {
        let base = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&base);
        let project = base.join("proyecto");
        std::fs::create_dir_all(&project).unwrap();
        (Window { base: Some(base.join("datos")), ..Window::default() }, project)
    }

    fn run(window: &Window, project: &std::path::Path, name: &str, arguments: Value) -> Result<Said, String> {
        window.ask(&scope(project), name, arguments)
    }

    #[test]
    fn a_skill_is_written_listed_read_turned_off_and_removed() {
        let (window, project) = window("sens-tools-kit-skill");
        let body = json!({ "name": "release-notes", "description": "Writes the release notes", "body": "# Notas\n\nUna línea por cambio." });
        assert!(run(&window, &project, "create_skill", body).unwrap() == Said::Text(format!("The skill release-notes is written and on for this project. {NEXT}")));
        let Said::Text(listed) = run(&window, &project, "list_capabilities", json!({})).unwrap() else { panic!() };
        assert!(listed.contains("  release-notes · on · Writes the release notes"), "{listed}");
        let Said::Text(read) = run(&window, &project, "read_skill", json!({ "name": "release-notes" })).unwrap() else { panic!() };
        assert!(read.contains("Una línea por cambio."));
        run(&window, &project, "set_capability", json!({ "kind": "skill", "name": "release-notes", "enabled": false })).unwrap();
        let Said::Text(listed) = run(&window, &project, "list_capabilities", json!({})).unwrap() else { panic!() };
        assert!(listed.contains("release-notes · off"));
        run(&window, &project, "remove_capability", json!({ "kind": "skill", "name": "release-notes" })).unwrap();
        let Said::Text(listed) = run(&window, &project, "list_capabilities", json!({})).unwrap() else { panic!() };
        assert!(!listed.contains("release-notes"));
        assert!(window.asked.lock().unwrap().iter().all(|(act, _)| act == "refresh_capabilities"));
    }

    #[test]
    fn a_server_is_added_without_secrets() {
        let (window, project) = window("sens-tools-kit-server");
        run(&window, &project, "add_server", json!({ "name": "docs", "command": "npx", "args": ["-y", "@x/docs"] })).unwrap();
        let Said::Text(listed) = run(&window, &project, "list_capabilities", json!({})).unwrap() else { panic!() };
        assert!(listed.contains("  docs · on · npx -y @x/docs"), "{listed}");
        assert!(run(&window, &project, "set_capability", json!({ "kind": "agent", "name": "x", "enabled": true })).unwrap_err().contains("not a skill"));
    }
}
