use std::path::PathBuf;

use serde_json::{Value, json};

use super::{Desk, Level, Said, Scope, Tool, schema, text};
use crate::{artifacts, canon, language, look, news, profile, projects};

const VIEWS: [&str; 5] = ["chat", "capabilities", "artifacts", "news", "settings"];
const MODES: [&str; 3] = ["dark", "light", "system"];
const ACCENTS: [&str; 5] = ["signal", "ice", "iris", "rose", "neutral"];
const LANGUAGES: [&str; 6] = ["en", "es", "fr", "de", "ja", "zh"];
const PREFERENCES: [&str; 4] = ["notify", "keep_in_tray", "start_with_windows", "check_updates"];
const MOST_CHARS: usize = 30_000;

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "get_settings",
        title: "Get settings",
        level: Level::Read,
        description: "How Sens is set up: the person's name, look, language, notices, tray, start with Windows and update checks.",
        input: || schema(json!({}), &[]),
        run: get_settings,
    },
    Tool {
        name: "set_look",
        title: "Set look",
        level: Level::Change,
        description: "Change how Sens looks: dark, light or as Windows, and its accent.",
        input: || schema(json!({ "mode": { "type": "string", "enum": MODES }, "accent": { "type": "string", "enum": ACCENTS } }), &[]),
        run: set_look,
    },
    Tool {
        name: "set_language",
        title: "Set language",
        level: Level::Change,
        description: "Change the language Sens speaks: English, Spanish, French, German, Japanese or Chinese.",
        input: || schema(json!({ "language": { "type": "string", "enum": LANGUAGES } }), &["language"]),
        run: set_language,
    },
    Tool {
        name: "set_preference",
        title: "Set preference",
        level: Level::Change,
        description: "Turn one of Sens's general preferences on or off: notices when a session ends while the person is away, staying in the tray when closed, starting with Windows, or checking for updates.",
        input: || schema(json!({ "name": { "type": "string", "enum": PREFERENCES }, "on": { "type": "boolean" } }), &["name", "on"]),
        run: set_preference,
    },
    Tool {
        name: "show_view",
        title: "Show view",
        level: Level::Show,
        description: "Bring one of Sens's views over the chat for the person: capabilities, artifacts, news or settings, or back to the chat.",
        input: || schema(json!({ "view": { "type": "string", "enum": VIEWS } }), &["view"]),
        run: show_view,
    },
    Tool {
        name: "check_updates",
        title: "Check updates",
        level: Level::Read,
        description: "Whether a newer Sens is out. Installing it is the person's click, since it restarts the app.",
        input: || schema(json!({}), &[]),
        run: check_updates,
    },
    Tool {
        name: "read_news",
        title: "Read news",
        level: Level::Read,
        description: "What changed in the versions of Sens the person has not read about yet.",
        input: || schema(json!({}), &[]),
        run: read_news,
    },
    Tool {
        name: "list_artifacts",
        title: "List artifacts",
        level: Level::Read,
        description: "What the sessions of every project made or linked: pictures, files and pages, newest first, with the session each came from.",
        input: || schema(json!({}), &[]),
        run: list_artifacts,
    },
    Tool {
        name: "read_artifact",
        title: "Read artifact",
        level: Level::Read,
        description: "The text of an artifact file, by the target list_artifacts gives.",
        input: || schema(json!({ "target": { "type": "string" } }), &["target"]),
        run: read_artifact,
    },
    Tool {
        name: "canon_status",
        title: "Canon status",
        level: Level::Read,
        description: "What Sens's Canon holds for this project: its rules, the changes it is holding back, the exceptions the person granted, and what it avoided. Only the person can accept, undo or change any of it.",
        input: || schema(json!({}), &[]),
        run: canon_status,
    },
];

fn data(desk: &Desk) -> Result<PathBuf, String> {
    desk.sens.data().ok_or_else(|| "Sens could not read its own data folder".to_string())
}

fn chosen(arguments: &Value, key: &str, allowed: &[&str]) -> Result<Option<String>, String> {
    match arguments[key].as_str() {
        None => Ok(None),
        Some(value) if allowed.contains(&value) => Ok(Some(value.to_string())),
        Some(value) => Err(format!("{value} is not one of {}", allowed.join(", "))),
    }
}

fn get_settings(desk: &Desk, _: &Scope, _: &Value) -> Result<Said, String> {
    let base = data(desk)?;
    let person = profile::load(&base);
    let looks = serde_json::to_value(look::load(&base)).unwrap_or_default();
    let spoken = language::load(&base).map(|spoken| serde_json::to_value(spoken).unwrap_or_default()).unwrap_or(json!("as Windows"));
    let on = |value: bool| if value { "on" } else { "off" };
    Ok([
        format!("Name: {}", person.name),
        format!("Look: {} with the {} accent", looks["mode"].as_str().unwrap_or("dark"), looks["accent"].as_str().unwrap_or("signal")),
        format!("Language: {}", spoken.as_str().unwrap_or_default()),
        format!("Notices: {}", on(person.notify)),
        format!("Keep in tray: {}", on(person.keep_in_tray)),
        format!("Start with Windows: {}", on(person.start_with_windows)),
        format!("Check for updates: {}", on(person.check_updates)),
    ]
    .join("\n")
    .into())
}

fn set_look(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let (mode, accent) = (chosen(arguments, "mode", &MODES)?, chosen(arguments, "accent", &ACCENTS)?);
    if mode.is_none() && accent.is_none() {
        return Err("say a `mode`, an `accent` or both".into());
    }
    desk.ui.act("set_look", json!({ "mode": mode, "accent": accent })).map(Said::from)
}

fn set_language(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let spoken = chosen(arguments, "language", &LANGUAGES)?.ok_or("`language` is missing")?;
    desk.ui.act("set_language", json!({ "language": spoken })).map(Said::from)
}

fn set_preference(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let name = chosen(arguments, "name", &PREFERENCES)?.ok_or("`name` is missing")?;
    let on = arguments["on"].as_bool().ok_or("`on` is missing")?;
    desk.ui.act("set_preference", json!({ "name": name, "on": on })).map(Said::from)
}

fn show_view(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let view = chosen(arguments, "view", &VIEWS)?.ok_or("`view` is missing")?;
    desk.ui.act("show_view", json!({ "session": scope.session, "view": view })).map(Said::from)
}

fn check_updates(desk: &Desk, _: &Scope, _: &Value) -> Result<Said, String> {
    desk.ui.act_within("check_updates", json!({}), std::time::Duration::from_secs(30)).map(Said::from)
}

fn read_news(_: &Desk, _: &Scope, _: &Value) -> Result<Said, String> {
    let all = news::since("")?;
    let said: Vec<String> = all.iter().take(5).map(|one| format!("{} · {} ({})\n{}", one.version, one.title, one.published, one.notes.trim())).collect();
    Ok(if said.is_empty() { "There is no news.".into() } else { said.join("\n\n").chars().take(MOST_CHARS).collect::<String>().into() })
}

fn list_artifacts(desk: &Desk, _: &Scope, _: &Value) -> Result<Said, String> {
    let all = artifacts::all(&projects::load(&data(desk)?));
    let lines: Vec<String> = all
        .iter()
        .take(100)
        .map(|one| {
            let kind = serde_json::to_value(one.kind).ok().and_then(|kind| kind.as_str().map(str::to_string)).unwrap_or_default();
            let from = one.session_title.as_deref().map(|title| format!(" · from \"{title}\"")).unwrap_or_default();
            format!("{kind} · {} · {} · {}{from}", one.name, one.project, one.target)
        })
        .collect();
    Ok(if lines.is_empty() { "No session has made an artifact yet.".into() } else { lines.join("\n").into() })
}

fn read_artifact(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let read = artifacts::text(&projects::load(&data(desk)?), &text(arguments, "target")?)?;
    Ok(read.chars().take(MOST_CHARS).collect::<String>().into())
}

fn canon_status(_: &Desk, scope: &Scope, _: &Value) -> Result<Said, String> {
    let work = scope.work().to_string_lossy().into_owned();
    let status = json!({
        "rules": canon::canon_rules(work.clone()),
        "held": canon::canon_held(work.clone()),
        "exceptions": canon::canon_exceptions(work.clone()),
        "avoided": canon::canon_avoided(work, 0),
    });
    let text = serde_json::to_string_pretty(&status).map_err(|error| error.to_string())?;
    Ok(text.chars().take(MOST_CHARS).collect::<String>().into())
}

#[cfg(test)]
mod tests {
    use super::super::call;
    use super::super::testing::{Window, scope};
    use super::*;
    use crate::terminal::Consoles;
    use sens_agent::canon::keeper::Keeper;

    fn run(window: &Window, name: &str, arguments: Value) -> Result<Said, String> {
        let consoles = Consoles::default();
        let keeper = Keeper::default();
        let here = std::env::temp_dir().join("sens-tools-app");
        std::fs::create_dir_all(&here).unwrap();
        call(&window.desk(&consoles, &keeper), &scope(&here), name, &arguments).unwrap()
    }

    #[test]
    fn settings_read_from_the_data_folder() {
        let base = std::env::temp_dir().join("sens-tools-app-settings");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        profile::rename(&base, "Sofía").unwrap();
        let window = Window { base: Some(base), ..Window::default() };
        let Said::Text(said) = run(&window, "get_settings", json!({})).unwrap() else { panic!() };
        assert!(said.starts_with("Name: Sofía\nLook: dark with the signal accent"), "{said}");
    }

    #[test]
    fn a_choice_must_be_one_sens_offers() {
        let window = Window::default();
        assert!(run(&window, "set_look", json!({ "accent": "purple" })).unwrap_err().contains("signal, ice, iris, rose, neutral"));
        assert!(run(&window, "set_look", json!({})).unwrap_err().contains("mode"));
        assert!(run(&window, "set_language", json!({ "language": "it" })).is_err());
        run(&window, "set_look", json!({ "mode": "light" })).unwrap();
        run(&window, "set_preference", json!({ "name": "keep_in_tray", "on": false })).unwrap();
        let asked = window.asked.lock().unwrap().clone();
        assert_eq!(asked[0], ("set_look".to_string(), json!({ "mode": "light", "accent": null })));
        assert_eq!(asked[1], ("set_preference".to_string(), json!({ "name": "keep_in_tray", "on": false })));
    }

    #[test]
    fn the_canon_is_only_read() {
        let window = Window::default();
        let Said::Text(said) = run(&window, "canon_status", json!({})).unwrap() else { panic!() };
        let status: Value = serde_json::from_str(&said).unwrap();
        assert!(["rules", "held", "exceptions", "avoided"].iter().all(|key| status.get(key).is_some()));
        assert!(TOOLS.iter().filter(|tool| tool.name.starts_with("canon")).all(|tool| tool.level == Level::Read));
    }
}
