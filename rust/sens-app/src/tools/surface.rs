use serde_json::{Value, json};

use super::{Desk, Level, Said, Scope, Tool, schema, text};

const PANES: [&str; 5] = ["files", "changes", "web", "terminal", "tasks"];
const NOTICE_TITLE: &str = "Claude";

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "open_file",
        title: "Open file in Sens",
        level: Level::Show,
        description: "Show a file of the project in Sens's file viewer, at a line if given, with the tree unfolded down to it. Use it to point the person at what you are talking about. It only changes the screen when the person is looking at this session.",
        input: || {
            schema(
                json!({
                    "path": { "type": "string", "description": "Path relative to the project root, or absolute inside it." },
                    "line": { "type": "integer", "minimum": 1, "description": "Line to bring into view." }
                }),
                &["path"],
            )
        },
        run: open_file,
    },
    Tool {
        name: "show_pane",
        title: "Show pane",
        level: Level::Show,
        description: "Open one of Sens's side panes for the person: files (tree and viewer), changes (the diff of the work), web (browser and preview), terminal, or tasks (what runs in the background). It only changes the screen when the person is looking at this session.",
        input: || schema(json!({ "pane": { "type": "string", "enum": PANES } }), &["pane"]),
        run: show_pane,
    },
    Tool {
        name: "close_pane",
        title: "Close pane",
        level: Level::Show,
        description: "Close Sens's side pane, giving the chat the room. It only changes the screen when the person is looking at this session.",
        input: || schema(json!({}), &[]),
        run: close_pane,
    },
    Tool {
        name: "open_terminal_tab",
        title: "Open terminal tab",
        level: Level::Show,
        description: "Open a new terminal tab for the person, in the session's folder, and show it. For running commands yourself use run_in_terminal.",
        input: || schema(json!({}), &[]),
        run: open_terminal_tab,
    },
    Tool {
        name: "get_layout",
        title: "Get layout",
        level: Level::Read,
        description: "What the person has on screen in Sens: whether they are looking at this session, which side pane is open, the file in the viewer, the terminals and how many chats are side by side.",
        input: || schema(json!({}), &[]),
        run: get_layout,
    },
    Tool {
        name: "screenshot_app",
        title: "Screenshot Sens",
        level: Level::Read,
        description: "A picture of Sens's window as the person sees it. Use it to check what the person is looking at or how something looks in the app.",
        input: || schema(json!({}), &[]),
        run: screenshot_app,
    },
    Tool {
        name: "notify",
        title: "Notify",
        level: Level::Show,
        description: "Send the person a Windows notification, for when they may be away: a long job done, or something that needs them.",
        input: || {
            schema(
                json!({
                    "body": { "type": "string", "description": "What to tell, in one or two short sentences." },
                    "title": { "type": "string", "description": "Its title (Claude if not said)." }
                }),
                &["body"],
            )
        },
        run: notify,
    },
];

fn shown(desk: &Desk, scope: &Scope, act: &str, input: Value) -> Result<Said, String> {
    let mut input = input;
    input["session"] = json!(scope.session);
    desk.ui.act(act, input).map(Said::from)
}

fn inside(scope: &Scope, path: &str) -> Result<String, String> {
    let missing = || format!("there is no file {path} in the project");
    let full = std::fs::canonicalize(scope.work().join(path)).map_err(|_| missing())?;
    let work = std::fs::canonicalize(scope.work()).map_err(|_| missing())?;
    let relative = full.strip_prefix(&work).map_err(|_| format!("{path} is outside this session's project"))?;
    if !full.is_file() {
        return Err(missing());
    }
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn open_file(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let path = inside(scope, &text(arguments, "path")?)?;
    shown(desk, scope, "open_file", json!({ "path": path, "line": arguments["line"].as_u64() }))
}

fn show_pane(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let pane = text(arguments, "pane")?;
    if !PANES.contains(&pane.as_str()) {
        return Err(format!("there is no {pane} pane; it is one of {}", PANES.join(", ")));
    }
    shown(desk, scope, "show_pane", json!({ "pane": pane }))
}

fn close_pane(desk: &Desk, scope: &Scope, _: &Value) -> Result<Said, String> {
    shown(desk, scope, "close_pane", json!({}))
}

fn open_terminal_tab(desk: &Desk, scope: &Scope, _: &Value) -> Result<Said, String> {
    shown(desk, scope, "open_terminal_tab", json!({ "root": scope.work() }))
}

fn get_layout(desk: &Desk, scope: &Scope, _: &Value) -> Result<Said, String> {
    shown(desk, scope, "get_layout", json!({}))
}

fn screenshot_app(desk: &Desk, _: &Scope, _: &Value) -> Result<Said, String> {
    let picture = desk.sens.screenshot().ok_or("Sens's window is minimized or hidden, so there is nothing to see")?;
    Ok(Said::Picture { media_type: picture.media_type, data: picture.data, caption: "Sens's window, as the person sees it.".into() })
}

fn notify(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let body = text(arguments, "body")?;
    let title = text(arguments, "title").unwrap_or_else(|_| NOTICE_TITLE.to_string());
    desk.sens.notify(&title, &body)?;
    Ok("The person was notified.".into())
}

#[cfg(test)]
mod tests {
    use super::super::call;
    use super::super::testing::{Window, scope};
    use super::*;
    use std::path::PathBuf;

    fn project() -> PathBuf {
        let root = std::env::temp_dir().join("sens-tools-surface");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/app.ts"), "export const a = 1;\n").unwrap();
        root
    }

    #[test]
    fn a_file_opens_by_its_path_in_the_project_and_nothing_outside_it() {
        let root = project();
        let window = Window::default();
        let desk = window.desk();
        let here = scope(&root);

        call(&desk, &here, "open_file", &json!({ "path": "src/app.ts", "line": 12 })).unwrap().unwrap();
        call(&desk, &here, "open_file", &json!({ "path": root.join("src/app.ts").to_string_lossy() })).unwrap().unwrap();
        let asked = window.asked.lock().unwrap().clone();
        assert_eq!(asked[0], ("open_file".to_string(), json!({ "path": "src/app.ts", "line": 12, "session": "s1" })));
        assert_eq!(asked[1].1["path"], "src/app.ts");

        assert!(call(&desk, &here, "open_file", &json!({ "path": "src/missing.ts" })).unwrap().unwrap_err().contains("no file"));
        let outside = std::env::temp_dir().join("elsewhere.txt");
        std::fs::write(&outside, "x").unwrap();
        assert!(call(&desk, &here, "open_file", &json!({ "path": outside.to_string_lossy() })).unwrap().unwrap_err().contains("outside"));
        assert!(call(&desk, &here, "open_file", &json!({ "path": "../elsewhere.txt" })).unwrap().unwrap_err().contains("outside"));
    }

    #[test]
    fn a_pane_is_one_sens_has() {
        let window = Window::default();
        let desk = window.desk();
        let here = scope(&project());
        call(&desk, &here, "show_pane", &json!({ "pane": "changes" })).unwrap().unwrap();
        assert!(call(&desk, &here, "show_pane", &json!({ "pane": "settings" })).unwrap().unwrap_err().contains("files, changes, web, terminal, tasks"));
        assert_eq!(window.asked.lock().unwrap()[0], ("show_pane".to_string(), json!({ "pane": "changes", "session": "s1" })));
    }

    #[test]
    fn the_window_is_seen_as_a_picture_unless_it_is_hidden() {
        let here = scope(&project());
        let window = Window::default();
        let said = call(&window.desk(), &here, "screenshot_app", &json!({})).unwrap().unwrap();
        assert!(matches!(said, Said::Picture { ref media_type, .. } if media_type == "image/png"));
        let hidden = Window { hidden: true, ..Window::default() };
        assert!(call(&hidden.desk(), &here, "screenshot_app", &json!({})).unwrap().unwrap_err().contains("hidden"));
    }

    #[test]
    fn a_notice_reaches_the_person_titled_by_claude_unless_told_otherwise() {
        let window = Window::default();
        let desk = window.desk();
        let here = scope(&project());
        call(&desk, &here, "notify", &json!({ "body": "Las pruebas pasan." })).unwrap().unwrap();
        call(&desk, &here, "notify", &json!({ "body": "Listo", "title": "Compilación" })).unwrap().unwrap();
        assert!(call(&desk, &here, "notify", &json!({})).unwrap().is_err());
        assert_eq!(*window.notified.lock().unwrap(), [("Claude".to_string(), "Las pruebas pasan.".to_string()), ("Compilación".to_string(), "Listo".to_string())]);
    }
}
