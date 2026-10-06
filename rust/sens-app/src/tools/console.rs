use std::time::Duration;

use serde_json::{Value, json};

use super::{Desk, Level, Scope, Tool, number, schema, text};
use crate::terminal::{Listed, Opened};

const WAIT: u64 = 120;
const LONGEST_WAIT: u64 = 600;
const LINES: u64 = 200;
const MOST_LINES: u64 = 1000;
const MOST_CHARS: usize = 30_000;

pub const TOOLS: &[Tool] = &[
    Tool {
        name: "run_in_terminal",
        title: "Run in terminal",
        level: Level::Change,
        description: "Run a command in a terminal of Sens, the only shell available to you. On Windows it runs in PowerShell, so write PowerShell, not bash. It starts in the session's working folder; each call is a fresh shell, so `cd` and variables do not carry over. In the foreground it waits for the command to end and returns its output and exit code. With `background` it returns at once and the command keeps running in a terminal tab the person can watch: use it for servers, watchers and anything long. A foreground command that outlives `wait` is not killed; it moves to the background the same way.",
        input: || {
            schema(
                json!({
                    "command": { "type": "string", "description": "The command, as typed at the prompt." },
                    "description": { "type": "string", "description": "What it does, in a few words, as the tab and the chat will title it." },
                    "background": { "type": "boolean", "description": "Return at once and leave it running in a terminal tab." },
                    "wait": { "type": "integer", "minimum": 1, "maximum": LONGEST_WAIT, "description": "Seconds to wait in the foreground before moving it to the background (120 if not said)." }
                }),
                &["command", "description"],
            )
        },
        run: run_in_terminal,
    },
    Tool {
        name: "read_terminal",
        title: "Read terminal",
        level: Level::Read,
        description: "Read what a terminal of Sens shows: one you started in the background, or one the person uses, with its prompts, the commands typed and what they printed. Use it to follow a background command, or when the person mentions something they ran or saw in their terminal. Without `terminal`, the one on screen.",
        input: || {
            schema(
                json!({
                    "terminal": { "type": "integer", "minimum": 1, "description": "Which terminal, by the number list_terminals or a run gave." },
                    "lines": { "type": "integer", "minimum": 1, "maximum": MOST_LINES, "description": "How many lines from the end (200 if not said)." }
                }),
                &[],
            )
        },
        run: read_terminal,
    },
    Tool {
        name: "write_terminal",
        title: "Type in terminal",
        level: Level::Change,
        description: "Type into a running terminal, as the person would: answer a prompt, send input to a program, or press Ctrl+C with \"\\u0003\". Enter is pressed after the text unless `enter` is false.",
        input: || {
            schema(
                json!({
                    "terminal": { "type": "integer", "minimum": 1 },
                    "text": { "type": "string", "description": "What to type." },
                    "enter": { "type": "boolean", "description": "Press Enter after it (true if not said)." }
                }),
                &["terminal", "text"],
            )
        },
        run: write_terminal,
    },
    Tool {
        name: "stop_terminal",
        title: "Stop terminal",
        level: Level::Change,
        description: "Stop what runs in a terminal, with every process it started. Its tab stays so its output can still be read.",
        input: || schema(json!({ "terminal": { "type": "integer", "minimum": 1 } }), &["terminal"]),
        run: stop_terminal,
    },
    Tool {
        name: "list_terminals",
        title: "List terminals",
        level: Level::Read,
        description: "The terminals open in this project: their number, shell, folder, what they run and whether it still runs.",
        input: || schema(json!({}), &[]),
        run: list_terminals,
    },
];

fn run_in_terminal(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<String, String> {
    let command = text(arguments, "command")?;
    let title = text(arguments, "description").unwrap_or_else(|_| command.lines().next().unwrap_or_default().to_string());
    let tell = desk.tell.clone();
    let opened = desk.consoles.run(scope.work(), &command, &title, move |heard| tell("terminal", json!(heard)))?;
    if arguments["background"].as_bool() == Some(true) {
        adopt(desk, scope, &opened, &title);
        return Ok(format!("Running in the background in terminal {}. Follow it with read_terminal and stop it with stop_terminal.", opened.id));
    }
    let wait = arguments["wait"].as_u64().unwrap_or(WAIT).clamp(1, LONGEST_WAIT);
    let ran = desk.consoles.wait(opened.id, Duration::from_secs(wait)).ok_or_else(|| format!("terminal {} vanished", opened.id))?;
    match ran.code {
        Some(code) => {
            let _ = desk.consoles.close(opened.id);
            let ending = code.map_or("Ended without an exit code".to_string(), |code| format!("Exit code {code}"));
            Ok(format!("{ending}\n{}", tail(&ran.text)))
        }
        None => {
            adopt(desk, scope, &opened, &title);
            Ok(format!(
                "Still running after {wait} s, so it moved to the background in terminal {}. Follow it with read_terminal and stop it with stop_terminal. So far:\n{}",
                opened.id,
                tail(&ran.text)
            ))
        }
    }
}

fn adopt(desk: &Desk, scope: &Scope, opened: &Opened, title: &str) {
    let backlog = desk.consoles.show(opened.id).unwrap_or_default();
    (desk.tell)(
        "terminal-adopted",
        json!({ "id": opened.id, "root": scope.work(), "shell": opened.shell, "title": title, "backlog": backlog }),
    );
}

fn tail(text: &str) -> String {
    if text.trim().is_empty() {
        return "(no output)".into();
    }
    let over = text.len().saturating_sub(MOST_CHARS);
    if over == 0 {
        return text.to_string();
    }
    let cut = (over..text.len()).find(|at| text.is_char_boundary(*at)).unwrap_or(over);
    format!("[the first {cut} bytes are cut]\n{}", &text[cut..])
}

fn ours(desk: &Desk, scope: &Scope, terminal: u64) -> Result<Listed, String> {
    desk.consoles
        .listed()
        .into_iter()
        .find(|one| u64::from(one.id) == terminal)
        .filter(|one| scope.holds(&one.cwd))
        .ok_or_else(|| format!("there is no terminal {terminal} in this project; list_terminals shows the open ones"))
}

fn read_terminal(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<String, String> {
    let lines = arguments["lines"].as_u64().unwrap_or(LINES).clamp(1, MOST_LINES);
    let terminal = arguments["terminal"].as_u64();
    if let Some(terminal) = terminal {
        let one = ours(desk, scope, terminal)?;
        if let Some(text) = desk.consoles.last_lines(one.id, lines as usize) {
            let state = if one.running { "still running" } else { "ended" };
            return Ok(format!("Terminal {} · {} · {state}\n{}", one.id, one.title.unwrap_or(one.shell), tail(&text)));
        }
    }
    desk.ui.act("read_terminal", json!({ "terminal": terminal, "lines": lines, "within": scope.within }))
}

fn write_terminal(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<String, String> {
    let one = ours(desk, scope, number(arguments, "terminal")?)?;
    let typed = arguments["text"].as_str().ok_or("`text` is missing")?;
    let enter = if arguments["enter"].as_bool() == Some(false) { "" } else { "\r" };
    desk.consoles.write(one.id, &format!("{typed}{enter}"))?;
    Ok(format!("Typed into terminal {}. Read it with read_terminal to see what it answered.", one.id))
}

fn stop_terminal(desk: &Desk, scope: &Scope, arguments: &Value) -> Result<String, String> {
    let one = ours(desk, scope, number(arguments, "terminal")?)?;
    desk.consoles.stop(one.id)?;
    Ok(format!("Stopped terminal {}.", one.id))
}

fn list_terminals(desk: &Desk, scope: &Scope, _: &Value) -> Result<String, String> {
    let open: Vec<String> = desk
        .consoles
        .listed()
        .into_iter()
        .filter(|one| scope.holds(&one.cwd))
        .map(|one| {
            let state = if one.running { "running" } else { "ended" };
            let what = one.title.map_or("the person's shell".to_string(), |title| format!("yours: {title}"));
            format!("{} · {} · {} · {what} · {state}", one.id, one.shell, one.cwd.display())
        })
        .collect();
    Ok(if open.is_empty() { "No terminal is open in this project.".into() } else { open.join("\n") })
}

#[cfg(test)]
mod tests {
    use super::super::testing::{Window, scope};
    use super::super::call;
    use super::*;
    use crate::terminal::Consoles;
    use sens_agent::canon::keeper::Keeper;
    use std::path::Path;

    #[test]
    fn long_output_keeps_its_end() {
        assert_eq!(tail("  \n"), "(no output)");
        assert_eq!(tail("ok"), "ok");
        let long = format!("{}fin", "ñ".repeat(MOST_CHARS));
        let kept = tail(&long);
        assert!(kept.starts_with("[the first "));
        assert!(kept.ends_with("fin"));
        assert!(kept.len() < long.len());
    }

    #[test]
    fn a_terminal_outside_the_project_is_out_of_reach() {
        let consoles = Consoles::default();
        let keeper = Keeper::default();
        let window = Window::default();
        let desk = window.desk(&consoles, &keeper);
        let here = scope(&std::env::temp_dir());
        let refused = call(&desk, &here, "stop_terminal", &json!({ "terminal": 9 })).unwrap().unwrap_err();
        assert!(refused.contains("no terminal 9"));
        assert_eq!(call(&desk, &here, "list_terminals", &json!({})).unwrap(), Ok("No terminal is open in this project.".into()));
        assert!(call(&desk, &here, "run_in_terminal", &json!({})).unwrap().unwrap_err().contains("command"));
    }

    #[test]
    fn reading_the_persons_terminal_asks_the_window() {
        let consoles = Consoles::default();
        let keeper = Keeper::default();
        let window = Window::default();
        let desk = window.desk(&consoles, &keeper);
        let here = scope(Path::new("C:/demo"));
        call(&desk, &here, "read_terminal", &json!({ "lines": 5000 })).unwrap().unwrap();
        let asked = window.asked.lock().unwrap();
        assert_eq!(asked[0], ("read_terminal".to_string(), json!({ "terminal": null, "lines": MOST_LINES, "within": ["C:/demo"] })));
    }

    fn quoted(text: &str) -> String {
        if cfg!(windows) { format!("Write-Output '{text}'") } else { format!("echo '{text}'") }
    }

    #[test]
    #[ignore = "opens a real shell"]
    fn a_command_answers_with_its_output_and_exit_code() {
        let consoles = Consoles::default();
        let keeper = Keeper::default();
        let window = Window::default();
        let desk = window.desk(&consoles, &keeper);
        let here = scope(&std::env::temp_dir());
        let command = format!("{}; exit 3", quoted("año-42"));
        let said = call(&desk, &here, "run_in_terminal", &json!({ "command": command, "description": "eco" })).unwrap().unwrap();
        assert!(said.starts_with("Exit code 3"), "{said}");
        assert!(said.contains("año-42"), "{said}");
        assert!(consoles.listed().is_empty());
        assert!(window.told.lock().unwrap().is_empty());
    }

    #[test]
    #[ignore = "opens a real shell"]
    fn a_slow_command_moves_to_the_background_and_can_be_read_and_stopped() {
        let consoles = Consoles::default();
        let keeper = Keeper::default();
        let window = Window::default();
        let desk = window.desk(&consoles, &keeper);
        let here = scope(&std::env::temp_dir());
        let slow = if cfg!(windows) { format!("{}; Start-Sleep 30", quoted("despierto")) } else { format!("{}; sleep 30", quoted("despierto")) };
        let said = call(&desk, &here, "run_in_terminal", &json!({ "command": slow, "description": "lento", "wait": 3 })).unwrap().unwrap();
        assert!(said.contains("moved to the background in terminal 1"), "{said}");
        assert!(said.contains("despierto"), "{said}");
        let adopted = window.told.lock().unwrap().iter().find(|(event, _)| event == "terminal-adopted").cloned().unwrap();
        assert_eq!(adopted.1["title"], "lento");
        assert!(adopted.1["backlog"].as_str().unwrap().contains("despierto"));
        let read = call(&desk, &here, "read_terminal", &json!({ "terminal": 1 })).unwrap().unwrap();
        assert!(read.contains("still running") && read.contains("despierto"), "{read}");
        call(&desk, &here, "stop_terminal", &json!({ "terminal": 1 })).unwrap().unwrap();
        assert!(consoles.wait(1, Duration::from_secs(10)).unwrap().code.is_some());
        assert!(call(&desk, &here, "list_terminals", &json!({})).unwrap().unwrap().contains("ended"));
    }
}
