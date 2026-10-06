use std::time::Duration;

use serde_json::{Value, json};

use super::{Desk, Level, Said, Scope, Tool, schema, text};

const PAGE: &str = include_str!("../../ui/src/features/web/page.js");
const MOST_CHARS: usize = 30_000;
const LOADING: Duration = Duration::from_secs(20);
const MOST_LINES: usize = 200;
const KEYS: [(&str, u32, &str); 15] = [
    ("Enter", 13, "\r"),
    ("Tab", 9, ""),
    ("Escape", 27, ""),
    ("Backspace", 8, ""),
    ("Delete", 46, ""),
    ("Space", 32, " "),
    ("ArrowUp", 38, ""),
    ("ArrowDown", 40, ""),
    ("ArrowLeft", 37, ""),
    ("ArrowRight", 39, ""),
    ("Home", 36, ""),
    ("End", 35, ""),
    ("PageUp", 33, ""),
    ("PageDown", 34, ""),
    ("F5", 116, ""),
];

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "navigate",
        title: "Navigate",
        level: Level::Show,
        description: "Open a page in Sens's browser, in the web pane beside the chat, and wait for it to load. It takes an address (https://…), a local server (localhost:5173), a page of the project (index.html, opened through Sens's preview server), words to search, or back, forward and reload. For a dev server, start it with run_in_terminal in the background first.",
        input: || schema(json!({ "url": { "type": "string" } }), &["url"]),
        run: navigate,
    },
    Tool {
        name: "read_page",
        title: "Read page",
        level: Level::Read,
        description: "The page in Sens's browser as a tree of what can be used on it, each with a reference (ref_N) that click, type_text and scroll take. With `all`, headings, landmarks and lists too. Prefer it to a screenshot for knowing what is on the page.",
        input: || schema(json!({ "all": { "type": "boolean", "description": "Include the page's structure, not only what can be used." } }), &[]),
        run: read_page,
    },
    Tool {
        name: "find",
        title: "Find on page",
        level: Level::Read,
        description: "Elements of the page whose name, role or placeholder contain the words, with their references. Up to 20.",
        input: || schema(json!({ "query": { "type": "string" } }), &["query"]),
        run: find,
    },
    Tool {
        name: "page_text",
        title: "Page text",
        level: Level::Read,
        description: "The text of the page in Sens's browser, its main content first, with its title and address.",
        input: || schema(json!({}), &[]),
        run: page_text,
    },
    Tool {
        name: "click",
        title: "Click",
        level: Level::Change,
        description: "Click an element of the page by its reference from read_page or find, or a point by its x and y in the page's CSS pixels.",
        input: || {
            schema(
                json!({
                    "ref": { "type": "string", "description": "ref_N from read_page or find." },
                    "x": { "type": "number" },
                    "y": { "type": "number" },
                    "double": { "type": "boolean", "description": "Double click." }
                }),
                &[],
            )
        },
        run: click,
    },
    Tool {
        name: "type_text",
        title: "Type text",
        level: Level::Change,
        description: "Type into the page as a person would: into the field given by its reference, or wherever the focus is. With `clear`, the field is emptied first. Press Enter with press_key.",
        input: || {
            schema(
                json!({
                    "text": { "type": "string" },
                    "ref": { "type": "string", "description": "The field, ref_N from read_page or find." },
                    "clear": { "type": "boolean" }
                }),
                &["text"],
            )
        },
        run: type_text,
    },
    Tool {
        name: "press_key",
        title: "Press key",
        level: Level::Change,
        description: "Press a key on the page: Enter, Tab, Escape, Backspace, Delete, Space, the arrows, Home, End, PageUp, PageDown or F5.",
        input: || schema(json!({ "key": { "type": "string", "enum": KEYS.map(|(key, _, _)| key) } }), &["key"]),
        run: press_key,
    },
    Tool {
        name: "scroll",
        title: "Scroll",
        level: Level::Show,
        description: "Scroll the page: to an element by its reference, or by screens (negative goes up).",
        input: || schema(json!({ "ref": { "type": "string" }, "screens": { "type": "number", "description": "How many screens; 1 if not said." } }), &[]),
        run: scroll,
    },
    Tool {
        name: "screenshot_page",
        title: "Screenshot page",
        level: Level::Read,
        description: "A picture of the page in Sens's browser as it is drawn now. Use it for how something looks; read_page tells what is there more cheaply.",
        input: || schema(json!({}), &[]),
        run: screenshot_page,
    },
    Tool {
        name: "eval_js",
        title: "Run JavaScript",
        level: Level::Change,
        description: "Run JavaScript in the page and get its value back as JSON; a promise is awaited. For inspecting and debugging the page, not for building it.",
        input: || schema(json!({ "expression": { "type": "string" } }), &["expression"]),
        run: eval_js,
    },
    Tool {
        name: "console_logs",
        title: "Console",
        level: Level::Read,
        description: "What the page in Sens's browser wrote to its console since it loaded, uncaught errors included.",
        input: || schema(json!({ "errors_only": { "type": "boolean" }, "pattern": { "type": "string", "description": "Only lines containing this." } }), &[]),
        run: console_logs,
    },
    Tool {
        name: "network_requests",
        title: "Network",
        level: Level::Read,
        description: "The requests the page in Sens's browser made, newest last, with their method, status and type, or why they failed.",
        input: || schema(json!({ "failed_only": { "type": "boolean" }, "pattern": { "type": "string", "description": "Only addresses containing this." } }), &[]),
        run: network_requests,
    },
    Tool {
        name: "resize_browser",
        title: "Resize browser",
        level: Level::Show,
        description: "Draw the page at a width, to check a layout as a phone (375) or a tablet (768) would show it; 0 goes back to the pane's own width.",
        input: || schema(json!({ "width": { "type": "integer", "minimum": 0, "maximum": 3840 } }), &["width"]),
        run: resize_browser,
    },
];

fn present(desk: &Desk, scope: &Scope) -> Result<(), String> {
    desk.ui.act("present", json!({ "session": scope.session })).map(|_| ())
}

fn evaluate(desk: &Desk, expression: &str) -> Result<Value, String> {
    let answer = desk.sens.devtools("Runtime.evaluate", json!({ "expression": expression, "returnByValue": true, "awaitPromise": true }))?;
    if let Some(thrown) = answer.get("exceptionDetails") {
        let reason = thrown["exception"]["description"].as_str().or(thrown["text"].as_str()).unwrap_or("the page threw");
        return Err(reason.lines().next().unwrap_or(reason).trim_start_matches("Error: ").to_string());
    }
    Ok(answer["result"]["value"].clone())
}

fn page(desk: &Desk, call: &str) -> Result<Value, String> {
    evaluate(desk, &format!("{PAGE}\n{call}"))
}

fn page_said(desk: &Desk, call: &str) -> Result<Said, String> {
    page(desk, call).map(|value| value.as_str().unwrap_or_default().to_string().into())
}

fn navigate(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let url = text(arguments, "url")?;
    desk.ui.act_within("browse", json!({ "session": scope.session, "url": url }), LOADING).map(Said::from)
}

fn read_page(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let interactive = arguments["all"].as_bool() != Some(true);
    page_said(desk, &format!("__sens.read({interactive})"))
}

fn find(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    page_said(desk, &format!("__sens.find({})", json!(text(arguments, "query")?)))
}

fn page_text(desk: &Desk, _: &Scope, _: &Value) -> Result<Said, String> {
    page_said(desk, "__sens.text()")
}

fn mouse(desk: &Desk, kind: &str, x: f64, y: f64, clicks: u64) -> Result<Value, String> {
    desk.sens.devtools("Input.dispatchMouseEvent", json!({ "type": kind, "x": x, "y": y, "button": "left", "clickCount": clicks }))
}

fn click(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    present(desk, scope)?;
    let (x, y, name) = match (arguments["ref"].as_str(), arguments["x"].as_f64(), arguments["y"].as_f64()) {
        (Some(reference), _, _) => {
            let spot = page(desk, &format!("__sens.spot({})", json!(reference)))?;
            (spot["x"].as_f64().unwrap_or_default(), spot["y"].as_f64().unwrap_or_default(), format!("\"{}\"", spot["name"].as_str().unwrap_or(reference)))
        }
        (None, Some(x), Some(y)) => (x, y, format!("{x}, {y}")),
        _ => return Err("say what to click: `ref`, or `x` and `y`".into()),
    };
    let clicks = if arguments["double"].as_bool() == Some(true) { 2 } else { 1 };
    mouse(desk, "mouseMoved", x, y, 0)?;
    for count in 1..=clicks {
        mouse(desk, "mousePressed", x, y, count)?;
        mouse(desk, "mouseReleased", x, y, count)?;
    }
    Ok(format!("Clicked {name}.").into())
}

fn type_text(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    present(desk, scope)?;
    let typed = arguments["text"].as_str().ok_or("`text` is missing")?;
    let into = match arguments["ref"].as_str() {
        Some(reference) => {
            let clear = arguments["clear"].as_bool() == Some(true);
            let name = page(desk, &format!("__sens.focus({}, {clear})", json!(reference)))?;
            format!(" into \"{}\"", name.as_str().unwrap_or(reference))
        }
        None => String::new(),
    };
    desk.sens.devtools("Input.insertText", json!({ "text": typed }))?;
    Ok(format!("Typed{into}.").into())
}

fn press_key(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    present(desk, scope)?;
    let wanted = text(arguments, "key")?;
    let (key, code, typed) = KEYS.iter().find(|(key, _, _)| key.eq_ignore_ascii_case(&wanted)).ok_or_else(|| format!("{wanted} is not a key press_key knows"))?;
    let named = if *key == "Space" { " " } else { key };
    for kind in ["keyDown", "keyUp"] {
        let mut event = json!({ "type": kind, "key": named, "code": key, "windowsVirtualKeyCode": code, "nativeVirtualKeyCode": code });
        if kind == "keyDown" && !typed.is_empty() {
            event["text"] = json!(typed);
        }
        desk.sens.devtools("Input.dispatchKeyEvent", event)?;
    }
    Ok(format!("Pressed {key}.").into())
}

fn scroll(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    present(desk, scope)?;
    let screens = arguments["screens"].as_f64().unwrap_or(1.0);
    page_said(desk, &format!("__sens.scroll({}, {screens})", json!(arguments["ref"].as_str())))
}

fn screenshot_page(desk: &Desk, _: &Scope, _: &Value) -> Result<Said, String> {
    let shot = desk.sens.devtools("Page.captureScreenshot", json!({ "format": "png" }))?;
    let data = shot["data"].as_str().ok_or("the browser gave no picture")?;
    Ok(Said::Picture { media_type: "image/png".into(), data: data.to_string(), caption: "The page in Sens's browser.".into() })
}

fn eval_js(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    present(desk, scope)?;
    let value = evaluate(desk, &text(arguments, "expression")?)?;
    let shown = match value {
        Value::Null => "undefined".to_string(),
        Value::String(text) => text,
        other => serde_json::to_string_pretty(&other).unwrap_or_default(),
    };
    Ok(shown.chars().take(MOST_CHARS).collect::<String>().into())
}

fn last<T>(all: Vec<T>) -> Vec<T> {
    let skip = all.len().saturating_sub(MOST_LINES);
    all.into_iter().skip(skip).collect()
}

fn console_logs(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let errors = arguments["errors_only"].as_bool() == Some(true);
    let pattern = arguments["pattern"].as_str().unwrap_or_default();
    let lines: Vec<String> = desk
        .sens
        .console()
        .into_iter()
        .filter(|(level, line)| (!errors || level == "error") && line.contains(pattern))
        .map(|(level, line)| format!("[{level}] {line}"))
        .collect();
    Ok(if lines.is_empty() { "The console has nothing that matches.".into() } else { last(lines).join("\n").into() })
}

fn network_requests(desk: &Desk, _: &Scope, arguments: &Value) -> Result<Said, String> {
    let failed = arguments["failed_only"].as_bool() == Some(true);
    let pattern = arguments["pattern"].as_str().unwrap_or_default();
    let lines: Vec<String> = desk
        .sens
        .requests()
        .into_iter()
        .filter(|request| request.url.contains(pattern))
        .filter(|request| !failed || request.failed.is_some() || request.status.is_some_and(|status| status >= 400))
        .map(|request| {
            let ending = match (&request.failed, request.status) {
                (Some(reason), _) => reason.clone(),
                (None, Some(status)) => status.to_string(),
                (None, None) => "pending".into(),
            };
            format!("{} {} · {ending} · {}", request.method, request.url, request.kind)
        })
        .collect();
    Ok(if lines.is_empty() { "No request matches.".into() } else { last(lines).join("\n").into() })
}

fn resize_browser(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<Said, String> {
    let width = arguments["width"].as_u64().ok_or("`width` is missing")?;
    desk.ui.act("browser_width", json!({ "session": scope.session, "width": width })).map(Said::from)
}

#[cfg(test)]
mod tests {
    use super::super::call;
    use super::super::testing::{Window, scope};
    use super::*;
    use crate::browser::Request;
    use crate::terminal::Consoles;
    use sens_agent::canon::keeper::Keeper;
    use std::path::Path;

    fn answering(answers: Vec<(&str, Result<Value, String>)>) -> Window {
        Window { answers: answers.into_iter().map(|(asked, answer)| (asked.to_string(), answer)).collect::<Vec<_>>().into(), ..Window::default() }
    }

    fn run(window: &Window, name: &str, arguments: Value) -> Result<Said, String> {
        let consoles = Consoles::default();
        let keeper = Keeper::default();
        call(&window.desk(&consoles, &keeper), &scope(Path::new("C:/demo")), name, &arguments).unwrap()
    }

    #[test]
    fn the_page_is_read_by_the_script_sens_puts_in_it() {
        let window = answering(vec![("__sens.find", Ok(json!({ "result": { "value": "button \"Comprar\" [ref_1]" } })))]);
        assert_eq!(run(&window, "find", json!({ "query": "comprar \"ya\"" })), Ok("button \"Comprar\" [ref_1]".into()));
        let (method, params) = window.called.lock().unwrap()[0].clone();
        assert_eq!(method, "Runtime.evaluate");
        let expression = params["expression"].as_str().unwrap();
        assert!(expression.starts_with("window.__sens = window.__sens ||"));
        assert!(expression.ends_with("__sens.find(\"comprar \\\"ya\\\"\")"));
    }

    #[test]
    fn a_click_goes_where_the_element_is_and_only_on_the_session_on_screen() {
        let window = answering(vec![("__sens.spot", Ok(json!({ "result": { "value": { "x": 40.5, "y": 12, "name": "Comprar" } } })))]);
        assert_eq!(run(&window, "click", json!({ "ref": "ref_1" })), Ok("Clicked \"Comprar\".".into()));
        let mice: Vec<(String, f64)> = window.called.lock().unwrap().iter().filter(|(method, _)| method == "Input.dispatchMouseEvent").map(|(_, event)| (event["type"].as_str().unwrap().to_string(), event["x"].as_f64().unwrap())).collect();
        assert_eq!(mice, [("mouseMoved".into(), 40.5), ("mousePressed".into(), 40.5), ("mouseReleased".into(), 40.5)]);
        assert_eq!(window.asked.lock().unwrap()[0], ("present".to_string(), json!({ "session": "s1" })));
        assert!(run(&window, "click", json!({})).unwrap_err().contains("ref"));
    }

    #[test]
    fn a_failure_in_the_page_comes_back_as_its_message() {
        let thrown = json!({ "exceptionDetails": { "text": "Uncaught", "exception": { "description": "Error: ref_9 is not on the page any more\n    at spot" } } });
        let window = answering(vec![("__sens.spot", Ok(thrown)), ("Input.insertText", Err("Sens's browser did not answer in time".into()))]);
        assert_eq!(run(&window, "click", json!({ "ref": "ref_9" })), Err("ref_9 is not on the page any more".into()));
        assert_eq!(run(&window, "type_text", json!({ "text": "hola" })), Err("Sens's browser did not answer in time".into()));
    }

    #[test]
    fn enter_is_pressed_with_its_text_so_a_form_is_sent() {
        let window = Window::default();
        assert_eq!(run(&window, "press_key", json!({ "key": "enter" })), Ok("Pressed Enter.".into()));
        let keys: Vec<Value> = window.called.lock().unwrap().iter().map(|(_, event)| event.clone()).collect();
        assert_eq!(keys[0]["text"], "\r");
        assert_eq!(keys[0]["windowsVirtualKeyCode"], 13);
        assert!(keys[1].get("text").is_none());
        assert!(run(&window, "press_key", json!({ "key": "F13" })).is_err());
    }

    #[test]
    fn javascript_answers_with_its_value() {
        let window = answering(vec![("Runtime.evaluate", Ok(json!({ "result": { "value": { "ok": true, "items": [1, 2] } } })))]);
        let said = run(&window, "eval_js", json!({ "expression": "fetch('/api').then(r => r.json())" })).unwrap();
        assert_eq!(said, Said::Text("{\n  \"items\": [\n    1,\n    2\n  ],\n  \"ok\": true\n}".into()));
        assert_eq!(window.called.lock().unwrap()[0].1["awaitPromise"], true);
    }

    #[test]
    fn the_console_and_the_network_are_filtered_as_asked() {
        let request = |url: &str, status: Option<u64>, failed: Option<&str>| Request { id: url.into(), method: "GET".into(), url: url.into(), kind: "Fetch".into(), status, failed: failed.map(str::to_string) };
        let window = Window {
            said: vec![("log".into(), "listo".into()), ("error".into(), "TypeError: x".into())],
            requested: vec![request("/api/a", Some(200), None), request("/api/b", Some(500), None), request("/app.js", None, Some("net::ERR_FAILED"))],
            ..Window::default()
        };
        assert_eq!(run(&window, "console_logs", json!({})), Ok("[log] listo\n[error] TypeError: x".into()));
        assert_eq!(run(&window, "console_logs", json!({ "errors_only": true })), Ok("[error] TypeError: x".into()));
        assert_eq!(run(&window, "network_requests", json!({ "failed_only": true })), Ok("GET /api/b · 500 · Fetch\nGET /app.js · net::ERR_FAILED · Fetch".into()));
        assert_eq!(run(&window, "network_requests", json!({ "pattern": "nada" })), Ok("No request matches.".into()));
    }

    #[test]
    fn a_screenshot_of_the_page_is_a_picture() {
        let window = answering(vec![("Page.captureScreenshot", Ok(json!({ "data": "iVBOR" })))]);
        assert!(matches!(run(&window, "screenshot_page", json!({})), Ok(Said::Picture { data, .. }) if data == "iVBOR"));
    }
}
