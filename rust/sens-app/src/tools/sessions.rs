use std::path::{Path, PathBuf};
use std::time::Duration;

use sens_agent::chat::Event;
use sens_agent::session::{self, Entry, Namer};
use serde_json::{Value, json};

use super::{Desk, Level, Said, Scope, Tool, ending, folded, schema, text};
use crate::projects;

const TURNS: u64 = 10;
const STARTING: Duration = Duration::from_secs(30);

fn project_field() -> Value {
    json!({ "type": "string", "description": "The project's folder, as list_projects gives it. This session's project if left out." })
}

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "list_projects",
        title: "List projects",
        level: Level::Read,
        description: "The projects open in Sens, newest first, with how many sessions each has and which one this session belongs to.",
        input: || schema(json!({}), &[]),
        run: list_projects,
    },
    Tool {
        name: "list_sessions",
        title: "List sessions",
        level: Level::Read,
        description: "The sessions (chats) of a project in Sens, newest first: their id, title, how many messages the person sent, and whether one is working now. Archived ones only when asked.",
        input: || schema(json!({ "project": project_field(), "archived": { "type": "boolean", "description": "Include the archived ones." } }), &[]),
        run: list_sessions,
    },
    Tool {
        name: "read_session",
        title: "Read session",
        level: Level::Read,
        description: "What was said in another session: the person's messages, Claude's answers and the tools it used, its last turns first cut.",
        input: || {
            schema(
                json!({
                    "session": { "type": "string", "description": "Its id, from list_sessions." },
                    "project": project_field(),
                    "turns": { "type": "integer", "minimum": 1, "description": "How many of the last turns (10 if not said)." }
                }),
                &["session"],
            )
        },
        run: read_session,
    },
    Tool {
        name: "open_session",
        title: "Open session",
        level: Level::Show,
        description: "Show a session to the person, beside this one, so the window splits in two chats.",
        input: || schema(json!({ "session": { "type": "string" }, "project": project_field() }), &["session"]),
        run: open_session,
    },
    Tool {
        name: "new_session",
        title: "New session",
        level: Level::Change,
        description: "Start a new session beside this one, in this project or another, and send it its first message so another Claude works on it in parallel. Without a prompt it only opens, empty, for the person.",
        input: || schema(json!({ "prompt": { "type": "string" }, "project": project_field() }), &[]),
        run: new_session,
    },
    Tool {
        name: "send_to_session",
        title: "Send to session",
        level: Level::Change,
        description: "Send a message to another session, as if the person typed it there; it opens beside this one if it is not on screen. Not to this session, and not to one that is still working.",
        input: || schema(json!({ "session": { "type": "string" }, "text": { "type": "string" }, "project": project_field() }), &["session", "text"]),
        run: send_to_session,
    },
    Tool {
        name: "stop_session",
        title: "Stop session",
        level: Level::Change,
        description: "Stop what another session is doing, as the stop button would. Not this one.",
        input: || schema(json!({ "session": { "type": "string" } }), &["session"]),
        run: stop_session,
    },
    Tool {
        name: "rename_session",
        title: "Rename session",
        level: Level::Change,
        description: "Give a session the title the list shows. This one if no session is said.",
        input: || schema(json!({ "title": { "type": "string" }, "session": { "type": "string" }, "project": project_field() }), &["title"]),
        run: rename_session,
    },
    Tool {
        name: "archive_session",
        title: "Archive session",
        level: Level::Change,
        description: "Move a session to the archive, out of the list, or back with `archived` false. Not this one.",
        input: || schema(json!({ "session": { "type": "string" }, "archived": { "type": "boolean" }, "project": project_field() }), &["session"]),
        run: archive_session,
    },
    Tool {
        name: "close_session_pane",
        title: "Close session pane",
        level: Level::Show,
        description: "Close the half of the window that shows a session, when two chats are side by side. The session stays in the list.",
        input: || schema(json!({ "session": { "type": "string" } }), &["session"]),
        run: close_session_pane,
    },
    Tool {
        name: "set_session_model",
        title: "Set model",
        level: Level::Change,
        description: "Choose the model, the effort and whether it thinks for the next messages of a session on screen, this one if none is said. The model is a name or id the model picker offers.",
        input: || {
            schema(
                json!({
                    "session": { "type": "string" },
                    "model": { "type": "string" },
                    "effort": { "type": "string", "enum": ["low", "medium", "high", "xhigh", "max"] },
                    "thinking": { "type": "boolean" }
                }),
                &[],
            )
        },
        run: set_session_model,
    },
];

fn home(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<PathBuf, String> {
    let Some(asked) = arguments["project"].as_str().filter(|asked| !asked.trim().is_empty()) else {
        return Ok(scope.root().to_path_buf());
    };
    if folded(asked) == folded(&scope.root().to_string_lossy()) {
        return Ok(scope.root().to_path_buf());
    }
    let known = desk.data().map(|data| projects::load(&data).projects).unwrap_or_default();
    known
        .into_iter()
        .find(|one| folded(&one.root) == folded(asked))
        .map(|one| PathBuf::from(one.root))
        .ok_or_else(|| format!("{asked} is not a project open in Sens; list_projects shows them"))
}

fn known(root: &Path, arguments: &Value) -> Result<String, String> {
    let id = text(arguments, "session")?;
    if !session::is_uuid(&id) || !session::exists(root, &id) {
        return Err(format!("there is no session {id} in {}; list_sessions shows them", root.display()));
    }
    Ok(id)
}

fn not_this(scope: &Scope, id: &str, what: &str) -> Result<(), String> {
    if id == scope.session {
        return Err(format!("this is your own session; you cannot {what} it"));
    }
    Ok(())
}

fn list_projects(desk: &Desk, scope: &Scope, _: &Value) -> Result<Said, String> {
    let data = desk.data()?;
    let here = folded(&scope.root().to_string_lossy());
    let lines: Vec<String> = projects::workspaces(&projects::load(&data))
        .into_iter()
        .map(|space| {
            let sessions = space.sessions.iter().filter(|one| !one.archived).count();
            let mark = if folded(&space.root) == here { " · this session's project" } else { "" };
            format!("{} · {} · {sessions} sessions{mark}", space.name, space.root)
        })
        .collect();
    Ok(if lines.is_empty() { "No project is open in Sens.".into() } else { lines.join("\n").into() })
}

fn list_sessions(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let root = home(desk, scope, arguments)?;
    let archived = arguments["archived"].as_bool() == Some(true);
    let lines: Vec<String> = session::list(&root)
        .into_iter()
        .filter(|one| archived || !one.archived)
        .map(|one| {
            let flags = [(one.id == scope.session, "this one"), (desk.sens.busy(&one.id), "working"), (one.archived, "archived")]
                .into_iter()
                .filter_map(|(on, flag)| on.then_some(flag))
                .collect::<Vec<_>>();
            let flags = if flags.is_empty() { String::new() } else { format!(" · {}", flags.join(", ")) };
            format!("{} · {} · {} messages{flags}", one.id, one.title, one.tasks)
        })
        .collect();
    Ok(if lines.is_empty() { format!("{} has no sessions.", root.display()).into() } else { lines.join("\n").into() })
}

fn transcript(entries: &[Entry], turns: usize) -> String {
    let mut all: Vec<Vec<String>> = vec![Vec::new()];
    for entry in entries {
        match entry {
            Entry::Task { text, files, .. } => {
                let attached = if files.is_empty() { String::new() } else { format!(" [attached: {}]", files.join(", ")) };
                all.push(vec![format!("Person: {}{attached}", text.trim())]);
            }
            Entry::Agent { event: Event::Said { text }, .. } => all.last_mut().into_iter().for_each(|turn| turn.push(format!("Claude: {}", text.trim()))),
            Entry::Agent { event: Event::Tool { name, .. }, .. } => all.last_mut().into_iter().for_each(|turn| turn.push(format!("  · used {name}"))),
            _ => {}
        }
    }
    let said: Vec<String> = all.into_iter().filter(|turn| !turn.is_empty()).map(|turn| turn.join("\n")).collect();
    let kept = &said[said.len().saturating_sub(turns)..];
    ending(&kept.join("\n\n"))
}

fn read_session(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let root = home(desk, scope, arguments)?;
    let id = known(&root, arguments)?;
    let turns = arguments["turns"].as_u64().unwrap_or(TURNS).max(1) as usize;
    let said = transcript(&session::read(&root, &id), turns);
    Ok(if said.is_empty() { "That session has nothing said yet.".into() } else { said.into() })
}

fn open_session(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let root = home(desk, scope, arguments)?;
    let id = known(&root, arguments)?;
    desk.ui.act("open_session", json!({ "session": scope.session, "target": id, "root": root })).map(Said::from)
}

fn new_session(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let root = home(desk, scope, arguments)?;
    let prompt = arguments["prompt"].as_str().map(str::trim).unwrap_or_default();
    desk.ui.act_within("new_session", json!({ "session": scope.session, "root": root, "prompt": prompt }), STARTING).map(Said::from)
}

fn send_to_session(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let root = home(desk, scope, arguments)?;
    let id = known(&root, arguments)?;
    not_this(scope, &id, "send a message to")?;
    if desk.sens.busy(&id) {
        return Err("that session is still working; wait until it is done, or stop it with stop_session".into());
    }
    let said = text(arguments, "text")?;
    desk.ui.act_within("send_to_session", json!({ "session": scope.session, "target": id, "root": root, "text": said }), STARTING).map(Said::from)
}

fn stop_session(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let id = text(arguments, "session")?;
    not_this(scope, &id, "stop")?;
    if !desk.sens.busy(&id) {
        return Ok("That session is not working.".into());
    }
    desk.ui.act("stop_session", json!({ "target": id })).map(Said::from)
}

fn rename_session(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let root = home(desk, scope, arguments)?;
    let id = if arguments["session"].as_str().is_some() { known(&root, arguments)? } else { scope.session.clone() };
    let title = session::entitle(&root, &id, &text(arguments, "title")?, Namer::User)?;
    let _ = desk.ui.act("refresh_sessions", json!({}));
    Ok(format!("The session is now called \"{title}\".").into())
}

fn archive_session(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let root = home(desk, scope, arguments)?;
    let id = known(&root, arguments)?;
    not_this(scope, &id, "archive")?;
    let archived = arguments["archived"].as_bool() != Some(false);
    session::archive(&root, &id, archived)?;
    let _ = desk.ui.act("refresh_sessions", json!({}));
    Ok((if archived { "The session is archived." } else { "The session is back in the list." }).into())
}

fn close_session_pane(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let id = text(arguments, "session")?;
    desk.ui.act("close_session_pane", json!({ "session": scope.session, "target": id })).map(Said::from)
}

fn set_session_model(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let target = arguments["session"].as_str().unwrap_or(&scope.session);
    let mut input = json!({ "target": target });
    for key in ["model", "effort", "thinking"] {
        input[key] = arguments[key].clone();
    }
    desk.ui.act("set_session_model", input).map(Said::from)
}

#[cfg(test)]
mod tests {
    use super::super::testing::Window;
    use super::*;

    const OTHER: &str = "0b9a3c1e-5d2f-4a6b-8c7d-1e2f3a4b5c6d";
    const THIS: &str = "1c2d3e4f-5a6b-4c7d-8e9f-0a1b2c3d4e5f";

    fn project(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        session::create(&root, OTHER, &[
            Entry::Opened { at: 1, root: root.to_string_lossy().into_owned() },
            Entry::Task { at: 2, text: "Arregla el login".into(), files: vec![], images: vec![] },
            Entry::Agent { at: 3, event: Event::Tool { id: "t".into(), name: "Read".into(), input: json!({}) } },
            Entry::Agent { at: 4, event: Event::Said { text: "Hecho: faltaba el token.".into() } },
            Entry::Task { at: 5, text: "¿Y las pruebas?".into(), files: vec!["src/login.ts".into()], images: vec![] },
            Entry::Agent { at: 6, event: Event::Said { text: "Pasan.".into() } },
        ])
        .unwrap();
        session::create(&root, THIS, &[Entry::Opened { at: 7, root: root.to_string_lossy().into_owned() }]).unwrap();
        root
    }

    fn scope_of(root: &Path) -> Scope {
        Scope { session: THIS.into(), within: vec![root.to_string_lossy().into_owned()] }
    }

    fn run(window: &Window, root: &Path, name: &str, arguments: Value) -> Result<Said, String> {
        window.ask(&scope_of(root), name, arguments)
    }

    #[test]
    fn a_session_reads_as_its_turns() {
        let root = project("sens-tools-read-session");
        let window = Window::default();
        let whole = "Person: Arregla el login\n  · used Read\nClaude: Hecho: faltaba el token.\n\nPerson: ¿Y las pruebas? [attached: src/login.ts]\nClaude: Pasan.";
        assert_eq!(run(&window, &root, "read_session", json!({ "session": OTHER })), Ok(whole.into()));
        assert_eq!(run(&window, &root, "read_session", json!({ "session": OTHER, "turns": 1 })), Ok("Person: ¿Y las pruebas? [attached: src/login.ts]\nClaude: Pasan.".into()));
        assert!(run(&window, &root, "read_session", json!({ "session": "nope" })).unwrap_err().contains("no session nope"));
    }

    #[test]
    fn the_list_marks_this_session_and_the_working_ones() {
        let root = project("sens-tools-list-sessions");
        let window = Window { working: vec![OTHER.into()], ..Window::default() };
        let Said::Text(listed) = run(&window, &root, "list_sessions", json!({})).unwrap() else { panic!() };
        assert!(listed.contains(&format!("{OTHER} · Arregla el login · 2 messages · working")), "{listed}");
        assert!(listed.contains(&format!("{THIS} · Empty session · 0 messages · this one")), "{listed}");
    }

    #[test]
    fn a_session_never_stops_archives_or_writes_to_itself_and_a_busy_one_is_left_alone() {
        let root = project("sens-tools-own-session");
        let window = Window { working: vec![OTHER.into()], ..Window::default() };
        assert!(run(&window, &root, "stop_session", json!({ "session": THIS })).unwrap_err().contains("your own session"));
        assert!(run(&window, &root, "archive_session", json!({ "session": THIS })).unwrap_err().contains("your own session"));
        assert!(run(&window, &root, "send_to_session", json!({ "session": THIS, "text": "hola" })).unwrap_err().contains("your own session"));
        assert!(run(&window, &root, "send_to_session", json!({ "session": OTHER, "text": "hola" })).unwrap_err().contains("still working"));
        run(&window, &root, "stop_session", json!({ "session": OTHER })).unwrap();
        assert_eq!(window.asked.lock().unwrap().last().unwrap(), &("stop_session".to_string(), json!({ "target": OTHER })));
    }

    #[test]
    fn rename_and_archive_land_on_disk_and_the_list_is_refreshed() {
        let root = project("sens-tools-rename-session");
        let window = Window::default();
        assert_eq!(run(&window, &root, "rename_session", json!({ "title": "Login" })), Ok("The session is now called \"Login\".".into()));
        assert!(session::list(&root).iter().any(|one| one.id == THIS && one.title == "Login"));
        run(&window, &root, "archive_session", json!({ "session": OTHER })).unwrap();
        assert!(session::list(&root).iter().any(|one| one.id == OTHER && one.archived));
        assert!(window.asked.lock().unwrap().iter().all(|(act, _)| act == "refresh_sessions"));
    }

    #[test]
    fn another_project_must_be_one_sens_knows() {
        let root = project("sens-tools-other-project");
        let window = Window::default();
        assert!(run(&window, &root, "list_sessions", json!({ "project": "C:/nowhere" })).unwrap_err().contains("not a project open in Sens"));
        assert!(run(&window, &root, "list_sessions", json!({ "project": root.to_string_lossy().to_uppercase() })).is_ok());
    }

    #[test]
    fn a_message_to_another_session_goes_through_the_window() {
        let root = project("sens-tools-send-session");
        let window = Window::default();
        run(&window, &root, "send_to_session", json!({ "session": OTHER, "text": "Revisa la rama" })).unwrap();
        let (act, input) = window.asked.lock().unwrap()[0].clone();
        assert_eq!(act, "send_to_session");
        assert_eq!((input["target"].as_str(), input["text"].as_str(), input["session"].as_str()), (Some(OTHER), Some("Revisa la rama"), Some(THIS)));
    }
}
