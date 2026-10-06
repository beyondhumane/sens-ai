use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime};

use sens_agent::said;
use serde::Serialize;
use serde_json::{Value, json};

use crate::tools::{self, SERVER, Said, Scope, Ui};

const PATH: &str = "/mcp";
const LATEST: &str = "2025-06-18";
const STALLED: Duration = Duration::from_secs(10);
const HEAD_CAP: usize = 16 * 1024;
const BODY_CAP: usize = 1024 * 1024;
const LATE: &str = "Sens did not answer in time: its window may be closed or busy.";

fn broken() -> String {
    said!(
        en: "the bridge to Claude Code broke",
        es: "el puente con Claude Code se rompió",
        fr: "le pont avec Claude Code est rompu",
        de: "die Brücke zu Claude Code ist abgebrochen",
        ja: "Claude Code との連携が切れました",
        zh: "与 Claude Code 的连接已中断",
    )
}

fn unopened(error: std::io::Error) -> String {
    said!(
        en: "couldn’t open the bridge to Claude Code: {error}",
        es: "no pude abrir el puente con Claude Code: {error}",
        fr: "impossible d’ouvrir le pont avec Claude Code : {error}",
        de: "die Brücke zu Claude Code konnte nicht geöffnet werden: {error}",
        ja: "Claude Code との連携を開始できませんでした: {error}",
        zh: "无法建立与 Claude Code 的连接：{error}",
    )
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Acting {
    pub ask: u64,
    pub act: String,
    pub input: Value,
}

pub type Ask = Arc<dyn Fn(Acting) + Send + Sync>;
type Answered = Option<Result<Said, String>>;
pub type Hand = Arc<dyn Fn(&Scope, &str, &Value, &dyn Ui) -> Answered + Send + Sync>;

#[derive(Default)]
struct Waiting {
    made: AtomicU64,
    answers: Mutex<HashMap<u64, mpsc::Sender<Result<String, String>>>>,
    scopes: Mutex<Vec<Scope>>,
}

impl Waiting {
    fn scope(&self, scope: Scope) -> Option<usize> {
        let mut scopes = self.scopes.lock().ok()?;
        Some(match scopes.iter().position(|known| *known == scope) {
            Some(at) => at,
            None => {
                scopes.push(scope);
                scopes.len() - 1
            }
        })
    }

    fn known(&self, at: usize) -> Option<Scope> {
        self.scopes.lock().ok()?.get(at).cloned()
    }

    fn act(&self, ask: &Ask, act: &str, input: Value, patience: Duration) -> Result<String, String> {
        let id = self.made.fetch_add(1, Ordering::SeqCst) + 1;
        let (answer, heard) = mpsc::channel();
        self.answers.lock().map_err(|_| broken())?.insert(id, answer);
        ask(Acting { ask: id, act: act.to_string(), input });
        let said = heard.recv_timeout(patience);
        if let Ok(mut answers) = self.answers.lock() {
            answers.remove(&id);
        }
        said.unwrap_or_else(|_| Err(LATE.to_string()))
    }

    fn answer(&self, ask: u64, said: Result<String, String>) -> Result<(), String> {
        let waiting = self.answers.lock().map_err(|_| broken())?.remove(&ask);
        if let Some(answer) = waiting {
            let _ = answer.send(said);
        }
        Ok(())
    }
}

struct Window<'a> {
    waiting: &'a Waiting,
    ask: &'a Ask,
}

impl Ui for Window<'_> {
    fn act_within(&self, act: &str, input: Value, patience: Duration) -> Result<String, String> {
        self.waiting.act(self.ask, act, input, patience)
    }
}

struct Served {
    port: u16,
    pass: String,
    waiting: Arc<Waiting>,
}

#[derive(Default)]
pub struct Bridge {
    open: Mutex<Option<Served>>,
}

impl Bridge {
    pub fn config(&self, scope: Scope, hands: impl FnOnce() -> (Ask, Hand)) -> Result<String, String> {
        let mut open = self.open.lock().map_err(|_| broken())?;
        if open.is_none() {
            let (ask, hand) = hands();
            *open = Some(start(ask, hand)?);
        }
        let served = open.as_ref().ok_or_else(broken)?;
        let scope = served.waiting.scope(scope).ok_or_else(broken)?;
        Ok(json!({
            "mcpServers": {
                SERVER: {
                    "type": "http",
                    "url": format!("http://127.0.0.1:{}{PATH}", served.port),
                    "headers": { "Authorization": format!("Bearer {}.{scope}", served.pass) }
                }
            }
        })
        .to_string())
    }

    pub fn answer(&self, ask: u64, said: Result<String, String>) -> Result<(), String> {
        let open = self.open.lock().map_err(|_| broken())?;
        match open.as_ref() {
            Some(served) => served.waiting.answer(ask, said),
            None => Ok(()),
        }
    }
}

fn start(ask: Ask, hand: Hand) -> Result<Served, String> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(unopened)?;
    let port = listener.local_addr().map_err(unopened)?.port();
    let pass = pass();
    let waiting = Arc::new(Waiting::default());
    let serving = waiting.clone();
    let checking = pass.clone();
    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let ask = ask.clone();
            let hand = hand.clone();
            let waiting = serving.clone();
            let pass = checking.clone();
            thread::spawn(move || {
                let _ = serve(stream, &pass, &waiting, &ask, &hand);
            });
        }
    });
    Ok(Served { port, pass, waiting })
}

fn scope_of(authorization: Option<&String>, pass: &str) -> Option<usize> {
    let (given, scope) = authorization?.strip_prefix("Bearer ")?.split_once('.')?;
    (given == pass).then(|| scope.parse().ok()).flatten()
}

fn pass() -> String {
    let now = SystemTime::now();
    format!("{:016x}{:016x}", RandomState::new().hash_one(now), RandomState::new().hash_one(now))
}

struct Asked {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

fn heard(stream: &TcpStream) -> std::io::Result<Option<Asked>> {
    let mut reader = BufReader::new(stream.try_clone()?).take(HEAD_CAP as u64);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    let mut parts = first.split_whitespace();
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else {
        return Ok(None);
    };
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let length: usize = headers.get("content-length").and_then(|value| value.parse().ok()).unwrap_or(0);
    if length > BODY_CAP {
        return Ok(None);
    }
    let mut body = vec![0; length];
    let mut reader = reader.into_inner();
    reader.read_exact(&mut body)?;
    Ok(Some(Asked {
        method: method.to_string(),
        path: target.split('?').next().unwrap_or_default().to_string(),
        headers,
        body,
    }))
}

fn serve(stream: TcpStream, pass: &str, waiting: &Waiting, ask: &Ask, hand: &Hand) -> std::io::Result<()> {
    stream.set_read_timeout(Some(STALLED))?;
    let Some(asked) = heard(&stream)? else {
        return reply(&stream, "400 Bad Request", None);
    };
    if asked.path != PATH {
        return reply(&stream, "404 Not Found", None);
    }
    if asked.headers.contains_key("origin") {
        return reply(&stream, "403 Forbidden", None);
    }
    let Some(scope) = scope_of(asked.headers.get("authorization"), pass).and_then(|at| waiting.known(at)) else {
        return reply(&stream, "401 Unauthorized", None);
    };
    if asked.method != "POST" {
        return reply(&stream, "405 Method Not Allowed", None);
    }
    let Ok(message) = serde_json::from_slice::<Value>(&asked.body) else {
        return reply(&stream, "400 Bad Request", Some(&failure(Value::Null, -32700, "Parse error")));
    };
    let window = Window { waiting, ask };
    match respond(&message, &|name, arguments| hand(&scope, name, arguments, &window)) {
        Some(answer) => reply(&stream, "200 OK", Some(&answer)),
        None => reply(&stream, "202 Accepted", None),
    }
}

fn reply(mut stream: &TcpStream, status: &str, body: Option<&Value>) -> std::io::Result<()> {
    let text = body.map(Value::to_string).unwrap_or_default();
    let kind = if body.is_some() { "Content-Type: application/json\r\n" } else { "" };
    let allow = if status.starts_with("405") { "Allow: POST\r\n" } else { "" };
    write!(stream, "HTTP/1.1 {status}\r\n{kind}{allow}Content-Length: {}\r\nConnection: close\r\n\r\n{text}", text.len())?;
    stream.flush()
}

fn failure(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn success(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn told(said: &Said, failed: bool) -> Value {
    json!({ "content": said.content(), "isError": failed })
}

fn respond(message: &Value, call: &dyn Fn(&str, &Value) -> Answered) -> Option<Value> {
    let id = message.get("id")?.clone();
    let params = &message["params"];
    Some(match message["method"].as_str().unwrap_or_default() {
        "initialize" => success(
            id,
            json!({
                "protocolVersion": params["protocolVersion"].as_str().unwrap_or(LATEST),
                "capabilities": { "tools": {} },
                "serverInfo": { "name": SERVER, "version": env!("CARGO_PKG_VERSION") }
            }),
        ),
        "ping" => success(id, json!({})),
        "tools/list" => success(id, json!({ "tools": tools::listed() })),
        "tools/call" => match call(params["name"].as_str().unwrap_or_default(), &params["arguments"]) {
            Some(Ok(said)) => success(id, told(&said, false)),
            Some(Err(reason)) => success(id, told(&Said::Text(reason), true)),
            None => failure(id, -32602, "Unknown tool"),
        },
        _ => failure(id, -32601, "Method not found"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(message: Value) -> Option<Value> {
        respond(&message, &|name, arguments| match name {
            "rm" => None,
            "broken" => Some(Err("`name` is missing".into())),
            _ => Some(Ok(format!("{name} {arguments}").into())),
        })
    }

    #[test]
    fn it_introduces_itself_in_the_version_it_was_asked_for() {
        let answer = call(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-03-26" } })).unwrap();
        assert_eq!(answer["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(answer["result"]["serverInfo"]["name"], SERVER);
        assert!(answer["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn it_offers_every_tool_of_sens() {
        let answer = call(json!({ "jsonrpc": "2.0", "id": "a", "method": "tools/list" })).unwrap();
        assert_eq!(answer["result"]["tools"], json!(tools::listed()));
        assert_eq!(answer["id"], "a");
    }

    #[test]
    fn a_call_answers_with_its_text_or_its_failure() {
        let asked = |name: &str| json!({ "jsonrpc": "2.0", "id": 8, "method": "tools/call", "params": { "name": name, "arguments": { "query": "tamaño" } } });
        let answer = call(asked("already_exists")).unwrap();
        assert_eq!(answer["result"]["content"][0]["text"], r#"already_exists {"query":"tamaño"}"#);
        assert_eq!(answer["result"]["isError"], false);
        let refused = call(asked("broken")).unwrap();
        assert_eq!(refused["result"]["isError"], true);
    }

    #[test]
    fn an_act_nobody_answers_says_so_as_an_error() {
        let waiting = Waiting::default();
        let ask: Ask = Arc::new(|_| {});
        let window = Window { waiting: &waiting, ask: &ask };
        assert_eq!(window.act("read_terminal", json!({})), Err(LATE.to_string()));
    }

    #[test]
    fn notifications_get_no_answer_and_the_unknown_gets_an_error() {
        assert_eq!(call(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })), None);
        assert_eq!(call(json!({ "jsonrpc": "2.0", "id": 4, "method": "resources/list" })).unwrap()["error"]["code"], -32601);
        assert_eq!(call(json!({ "jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": { "name": "rm" } })).unwrap()["error"]["code"], -32602);
    }

    fn post(port: u16, token: &str, extra: &str, body: &str) -> String {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
        write!(
            stream,
            "POST {PATH} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut said = String::new();
        stream.read_to_string(&mut said).unwrap();
        said
    }

    fn scope(within: &[&str]) -> Scope {
        Scope { session: "s1".into(), within: within.iter().map(|folder| folder.to_string()).collect() }
    }

    fn window_reader(bridge: Arc<Bridge>) -> impl FnOnce() -> (Ask, Hand) {
        move || {
            let ask: Ask = Arc::new(move |acting: Acting| {
                let bridge = bridge.clone();
                thread::spawn(move || bridge.answer(acting.ask, Ok(format!("pantalla {}", acting.input["lines"]))).unwrap());
            });
            let hand: Hand = Arc::new(|scope: &Scope, name: &str, arguments: &Value, ui: &dyn Ui| {
                (name == "read_terminal").then(|| ui.act(name, json!({ "lines": arguments["lines"], "within": scope.within }))).map(|said| said.map(|text| format!("{text} en {}", scope.within.join(" y ")).into()))
            });
            (ask, hand)
        }
    }

    #[test]
    #[ignore = "calls Claude Code, which calls the model"]
    fn claude_code_reads_the_terminal_through_the_bridge() {
        let bridge = Arc::new(Bridge::default());
        let answering = bridge.clone();
        let hands = move || {
            let ask: Ask = Arc::new(move |acting: Acting| {
                let _ = answering.answer(acting.ask, Ok("PS C:\\demo> npm run dev\nError: el puerto 5173 ya está en uso".into()));
            });
            let hand: Hand = Arc::new(|_: &Scope, name: &str, _: &Value, ui: &dyn Ui| (name == "read_terminal").then(|| ui.act(name, json!({})).map(Said::from)));
            (ask, hand)
        };
        let config = bridge.config(scope(&[&std::env::temp_dir().to_string_lossy()]), hands).unwrap();
        let mut claude = sens_agent::process::claude();
        claude
            .args(["-p", "--output-format", "json", "--model", "haiku", "--strict-mcp-config", "--mcp-config", &config, "--allowedTools", &tools::allowed()])
            .current_dir(std::env::temp_dir());
        let said = sens_agent::process::run(claude, "Lee mi terminal con la herramienta read_terminal y responde solo con el número de puerto del error.");
        assert!(said.as_deref().is_ok_and(|said| said.contains("5173")), "{said:?}");
    }

    #[test]
    fn over_http_it_acts_within_the_session_folders_and_turns_strangers_away() {
        let bridge = Arc::new(Bridge::default());
        let here: Value = serde_json::from_str(&bridge.config(scope(&["C:/demo"]), window_reader(bridge.clone())).unwrap()).unwrap();
        let there: Value = serde_json::from_str(&bridge.config(scope(&["C:/api", "C:/api/.sens/worktrees/ab12cd34"]), || unreachable!()).unwrap()).unwrap();
        let token = |config: &Value| config["mcpServers"][SERVER]["headers"]["Authorization"].as_str().unwrap().trim_start_matches("Bearer ").to_string();
        let url = here["mcpServers"][SERVER]["url"].as_str().unwrap().to_string();
        let port: u16 = url.trim_start_matches("http://127.0.0.1:").trim_end_matches(PATH).parse().unwrap();
        let call = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"read_terminal","arguments":{"lines":12}}}"#;

        let said = post(port, &token(&here), "", call);
        assert!(said.starts_with("HTTP/1.1 200 OK"), "{said}");
        assert!(said.contains(r#""text":"pantalla 12 en C:/demo""#), "{said}");
        assert!(post(port, &token(&there), "", call).contains(r#""text":"pantalla 12 en C:/api y C:/api/.sens/worktrees/ab12cd34""#));
        assert_eq!(there["mcpServers"][SERVER]["url"], url.as_str());

        let pass = token(&here).split_once('.').unwrap().0.to_string();
        for stranger in ["otro", pass.as_str(), &format!("{pass}.9"), "otro.0"] {
            assert!(post(port, stranger, "", call).starts_with("HTTP/1.1 401"), "{stranger}");
        }
        assert!(post(port, &token(&here), "Origin: https://evil.example\r\n", call).starts_with("HTTP/1.1 403"));
        assert!(post(port, &token(&here), "", r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#).starts_with("HTTP/1.1 202"));
        assert_eq!(bridge.config(scope(&["C:/demo"]), || unreachable!()).unwrap(), here.to_string());
    }
}
