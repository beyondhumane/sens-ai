use std::collections::VecDeque;
use std::sync::{Mutex, PoisonError};

use sens_agent::said;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::webview::{NewWindowResponse, PageLoadEvent};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Rect, Url, Webview, WebviewBuilder, WebviewUrl};

const LABEL: &str = "browser";
const HOST: &str = "main";
const EVENT: &str = "browser";
const KEPT: usize = 500;
const CLOSED: &str = "Nothing is open in Sens's browser; open a page with navigate first.";

#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub url: String,
    pub kind: String,
    pub status: Option<u64>,
    pub failed: Option<String>,
}

static SAID: Mutex<VecDeque<(String, String)>> = Mutex::new(VecDeque::new());
static REQUESTS: Mutex<VecDeque<Request>> = Mutex::new(VecDeque::new());

fn kept<T>(list: &Mutex<VecDeque<T>>, change: impl FnOnce(&mut VecDeque<T>)) {
    let mut list = list.lock().unwrap_or_else(PoisonError::into_inner);
    change(&mut list);
    while list.len() > KEPT {
        list.pop_front();
    }
}

fn remember(heard: &Heard) {
    match heard {
        Heard::Loading { .. } => kept(&SAID, VecDeque::clear),
        Heard::Said { level, text } => kept(&SAID, |said| said.push_back((level.clone(), text.clone()))),
        _ => {}
    }
}

pub fn said() -> Vec<(String, String)> {
    SAID.lock().unwrap_or_else(PoisonError::into_inner).iter().cloned().collect()
}

pub fn requests() -> Vec<Request> {
    REQUESTS.lock().unwrap_or_else(PoisonError::into_inner).iter().cloned().collect()
}

fn network(event: &str, value: &Value, list: &mut VecDeque<Request>) {
    let id = value["requestId"].as_str().unwrap_or_default();
    match event {
        "Network.requestWillBeSent" => list.push_back(Request {
            id: id.to_string(),
            method: value["request"]["method"].as_str().unwrap_or("GET").to_string(),
            url: value["request"]["url"].as_str().unwrap_or_default().to_string(),
            kind: value["type"].as_str().unwrap_or_default().to_string(),
            status: None,
            failed: None,
        }),
        "Network.responseReceived" => {
            if let Some(request) = list.iter_mut().rev().find(|request| request.id == id) {
                request.status = value["response"]["status"].as_u64();
            }
        }
        "Network.loadingFailed" => {
            if let Some(request) = list.iter_mut().rev().find(|request| request.id == id) {
                request.failed = Some(value["errorText"].as_str().unwrap_or("failed").to_string());
            }
        }
        _ => {}
    }
}

#[derive(Deserialize, Clone, Copy)]
pub struct Frame {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl Frame {
    fn rect(self) -> Rect {
        Rect {
            position: LogicalPosition::new(self.x, self.y).into(),
            size: LogicalSize::new(self.width.max(1.0), self.height.max(1.0)).into(),
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Heard {
    Loading { url: String },
    Loaded { url: String },
    Titled { title: String },
    Said { level: String, text: String },
}

fn tell(app: &AppHandle, heard: Heard) {
    remember(&heard);
    let _ = app.emit_to(HOST, EVENT, heard);
}

#[cfg(windows)]
pub fn devtools(app: &AppHandle, method: &str, params: &Value) -> Result<Value, String> {
    use std::sync::mpsc;
    use std::time::Duration;
    use webview2_com::CallDevToolsProtocolMethodCompletedHandler;
    use windows_core::HSTRING;

    const PATIENCE: Duration = Duration::from_secs(30);

    let view = view(app).ok_or(CLOSED)?;
    let (answer, heard) = mpsc::channel::<Result<String, String>>();
    let failing = answer.clone();
    let (method, params) = (HSTRING::from(method), HSTRING::from(params.to_string()));
    view.with_webview(move |platform| {
        let called = unsafe {
            platform.controller().CoreWebView2().and_then(|core| {
                let handler = CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |done, json| {
                    let _ = answer.send(match done {
                        Ok(()) => Ok(json),
                        Err(_) => Err(json),
                    });
                    Ok(())
                }));
                core.CallDevToolsProtocolMethod(&method, &params, &handler)
            })
        };
        if let Err(error) = called {
            let _ = failing.send(Err(error.message()));
        }
    })
    .map_err(failed)?;
    let json = heard.recv_timeout(PATIENCE).map_err(|_| "Sens's browser did not answer in time".to_string())?.map_err(|json| refusal(&json))?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

#[cfg(not(windows))]
pub fn devtools(app: &AppHandle, _method: &str, _params: &Value) -> Result<Value, String> {
    view(app).ok_or(CLOSED)?;
    Err("Sens's browser can only be driven on Windows".into())
}

fn refusal(json: &str) -> String {
    serde_json::from_str::<Value>(json).ok().and_then(|value| value["message"].as_str().map(str::to_string)).unwrap_or_else(|| json.to_string())
}

fn failed(error: tauri::Error) -> String {
    said!(
        en: "the browser didn’t respond: {error}",
        es: "el navegador no respondió: {error}",
        fr: "le navigateur n’a pas répondu : {error}",
        de: "der Browser hat nicht reagiert: {error}",
        ja: "ブラウザーが応答しませんでした: {error}",
        zh: "浏览器没有响应：{error}",
    )
}

fn view(app: &AppHandle) -> Option<Webview> {
    app.get_webview(LABEL)
}

pub fn allowed(url: &Url) -> bool {
    match url.scheme() {
        "http" | "https" => !url.host_str().is_some_and(|host| host.ends_with(".localhost")),
        "about" => true,
        _ => false,
    }
}

fn aimed(address: &str) -> Result<Url, String> {
    let url = Url::parse(address).map_err(|_| {
        said!(
            en: "{address} isn’t a valid address",
            es: "{address} no es una dirección válida",
            fr: "{address} n’est pas une adresse valide",
            de: "{address} ist keine gültige Adresse",
            ja: "{address} は有効なアドレスではありません",
            zh: "{address} 不是有效的地址",
        )
    })?;
    if !allowed(&url) {
        return Err(said!(
            en: "The browser only opens http and https addresses.",
            es: "El navegador solo abre direcciones http y https.",
            fr: "Le navigateur n’ouvre que les adresses http et https.",
            de: "Der Browser öffnet nur http- und https-Adressen.",
            ja: "ブラウザーで開けるのは http と https のアドレスだけです。",
            zh: "浏览器只能打开 http 和 https 地址。",
        ));
    }
    Ok(url)
}

pub fn open(app: &AppHandle, address: &str, frame: Frame, zoom: f64) -> Result<(), String> {
    let url = aimed(address)?;
    if let Some(view) = view(app) {
        view.navigate(url).map_err(failed)?;
        return place(app, frame, zoom);
    }

    let window = app.get_window(HOST).ok_or_else(|| {
        said!(
            en: "can’t find the Sens window",
            es: "no encuentro la ventana de Sens",
            fr: "fenêtre de Sens introuvable",
            de: "Sens-Fenster nicht gefunden",
            ja: "Sens のウィンドウが見つかりません",
            zh: "找不到 Sens 窗口",
        )
    })?;
    let loads = app.clone();
    let titles = app.clone();
    let popups = app.clone();
    let builder = WebviewBuilder::new(LABEL, WebviewUrl::External(url))
        .disable_drag_drop_handler()
        .on_navigation(allowed)
        .on_new_window(move |url, _| {
            if let Some(view) = view(&popups)
                && allowed(&url)
            {
                let _ = view.navigate(url);
            }
            NewWindowResponse::Deny
        })
        .on_page_load(move |_, payload| {
            let url = payload.url().to_string();
            tell(
                &loads,
                match payload.event() {
                    PageLoadEvent::Started => Heard::Loading { url },
                    PageLoadEvent::Finished => Heard::Loaded { url },
                },
            );
        })
        .on_document_title_changed(move |_, title| tell(&titles, Heard::Titled { title }));

    let rect = frame.rect();
    let view = window.add_child(builder, rect.position, rect.size).map_err(failed)?;
    view.set_zoom(zoom).map_err(failed)?;
    listen_console(&view, app.clone());
    Ok(())
}

pub fn place(app: &AppHandle, frame: Frame, zoom: f64) -> Result<(), String> {
    let Some(view) = view(app) else {
        return Ok(());
    };
    view.set_bounds(frame.rect()).map_err(failed)?;
    view.set_zoom(zoom).map_err(failed)
}

pub fn show(app: &AppHandle, shown: bool) -> Result<(), String> {
    let Some(view) = view(app) else {
        return Ok(());
    };
    match shown {
        true => view.show(),
        false => view.hide(),
    }
    .map_err(failed)
}

pub fn act(app: &AppHandle, act: &str) -> Result<(), String> {
    let Some(view) = view(app) else {
        return Ok(());
    };
    match act {
        "back" => view.eval("history.back()"),
        "forward" => view.eval("history.forward()"),
        "reload" => view.reload(),
        "close" => view.close(),
        other => {
            return Err(said!(
                en: "the browser doesn’t know the action {other}",
                es: "el navegador no sabe hacer {other}",
                fr: "le navigateur ne connaît pas l’action {other}",
                de: "der Browser kennt die Aktion {other} nicht",
                ja: "ブラウザーは「{other}」という操作に対応していません",
                zh: "浏览器不支持操作 {other}",
            ));
        }
    }
    .map_err(failed)
}

fn shown_value(value: &Value) -> String {
    match value.get("value") {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Null) | None => value
            .get("description")
            .or_else(|| value.get("unserializableValue"))
            .or_else(|| value.get("type"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        Some(other) => other.to_string(),
    }
}

fn console_line(called: &Value) -> Option<Heard> {
    let level = match called["type"].as_str()? {
        "warning" => "warn",
        "error" | "assert" => "error",
        "info" => "info",
        "debug" => "debug",
        _ => "log",
    };
    let text = called["args"]
        .as_array()?
        .iter()
        .map(shown_value)
        .collect::<Vec<_>>()
        .join(" ");
    Some(Heard::Said {
        level: level.into(),
        text,
    })
}

fn exception_line(thrown: &Value) -> Option<Heard> {
    let details = &thrown["exceptionDetails"];
    let text = details["exception"]["description"]
        .as_str()
        .or_else(|| details["text"].as_str())?;
    Some(Heard::Said {
        level: "error".into(),
        text: text.to_string(),
    })
}

#[cfg(windows)]
fn listen_console(view: &Webview, app: AppHandle) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2;
    use webview2_com::{CallDevToolsProtocolMethodCompletedHandler, DevToolsProtocolEventReceivedEventHandler, take_pwstr};
    use windows_core::{PWSTR, w};

    type Reader = fn(&Value) -> Option<Heard>;

    unsafe fn follow(core: &ICoreWebView2, event: &'static str, heard: impl Fn(&Value) + 'static) -> windows_core::Result<()> {
        let receiver = unsafe { core.GetDevToolsProtocolEventReceiver(&windows_core::HSTRING::from(event))? };
        let handler = DevToolsProtocolEventReceivedEventHandler::create(Box::new(move |_, args| {
            let Some(args) = args else {
                return Ok(());
            };
            let mut json = PWSTR::null();
            unsafe { args.ParameterObjectAsJson(&mut json)? };
            if let Ok(value) = serde_json::from_str::<Value>(&take_pwstr(json)) {
                heard(&value);
            }
            Ok(())
        }));
        let mut token = 0;
        unsafe { receiver.add_DevToolsProtocolEventReceived(&handler, &mut token) }
    }

    fn told(app: AppHandle, read: Reader) -> impl Fn(&Value) + 'static {
        move |value| {
            if let Some(heard) = read(value) {
                tell(&app, heard);
            }
        }
    }

    let _ = view.with_webview(move |platform| unsafe {
        let Ok(core) = platform.controller().CoreWebView2() else {
            return;
        };
        let _ = follow(&core, "Runtime.consoleAPICalled", told(app.clone(), console_line));
        let _ = follow(&core, "Runtime.exceptionThrown", told(app, exception_line));
        for event in ["Network.requestWillBeSent", "Network.responseReceived", "Network.loadingFailed"] {
            let _ = follow(&core, event, move |value| kept(&REQUESTS, |list| network(event, value, list)));
        }
        for domain in [w!("Runtime.enable"), w!("Network.enable")] {
            let _ = core.CallDevToolsProtocolMethod(domain, w!("{}"), &CallDevToolsProtocolMethodCompletedHandler::create(Box::new(|_, _| Ok(()))));
        }
    });
}

#[cfg(not(windows))]
fn listen_console(_view: &Webview, _app: AppHandle) {}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_agent::language::{Language, speaking};
    use serde_json::json;

    #[test]
    fn only_the_open_web_is_browsable() {
        let can = |address: &str| allowed(&Url::parse(address).unwrap());
        assert!(can("https://www.google.com/"));
        assert!(can("http://localhost:5173/app"));
        assert!(can("http://127.0.0.1:4321/abc/index.html"));
        assert!(can("about:blank"));
        assert!(!can("http://tauri.localhost/index.html"));
        assert!(!can("http://ipc.localhost/chat_send"));
        assert!(!can("file:///C:/Windows/win.ini"));
        assert!(!can("tauri://localhost/"));
        assert!(!can("javascript:alert(1)"));
    }

    #[test]
    fn an_address_that_is_not_web_is_refused_before_loading() {
        assert!(aimed("ftp://x.org").is_err());
        assert!(aimed("no es una url").is_err());
        assert_eq!(aimed("https://example.com").unwrap().as_str(), "https://example.com/");
    }

    #[test]
    fn a_refused_address_is_explained_in_the_language_spoken() {
        assert_eq!(aimed("ftp://x.org").unwrap_err(), "The browser only opens http and https addresses.");
        assert_eq!(speaking(Language::Es, || aimed("ftp://x.org")).unwrap_err(), "El navegador solo abre direcciones http y https.");
        assert_eq!(speaking(Language::Zh, || aimed("no es una url")).unwrap_err(), "no es una url 不是有效的地址");
    }

    #[test]
    fn a_console_call_reads_like_the_console() {
        let heard = console_line(&json!({
            "type": "warning",
            "args": [
                { "type": "string", "value": "cargados" },
                { "type": "number", "value": 3 },
                { "type": "object", "className": "Object", "description": "Object" },
                { "type": "undefined" }
            ]
        }));
        assert_eq!(heard, Some(Heard::Said { level: "warn".into(), text: "cargados 3 Object undefined".into() }));
    }

    #[test]
    fn a_request_is_followed_from_sent_to_answered_or_failed() {
        let mut list = VecDeque::new();
        network("Network.requestWillBeSent", &json!({ "requestId": "1", "type": "Fetch", "request": { "method": "POST", "url": "http://localhost:5173/api" } }), &mut list);
        network("Network.requestWillBeSent", &json!({ "requestId": "2", "type": "Script", "request": { "method": "GET", "url": "http://localhost:5173/app.js" } }), &mut list);
        network("Network.responseReceived", &json!({ "requestId": "1", "response": { "status": 500 } }), &mut list);
        network("Network.loadingFailed", &json!({ "requestId": "2", "errorText": "net::ERR_CONNECTION_REFUSED" }), &mut list);
        assert_eq!((list[0].method.as_str(), list[0].status, list[0].kind.as_str()), ("POST", Some(500), "Fetch"));
        assert_eq!(list[1].failed.as_deref(), Some("net::ERR_CONNECTION_REFUSED"));
        assert_eq!(refusal(r#"{"code":-32000,"message":"Cannot find context"}"#), "Cannot find context");
        assert_eq!(refusal("roto"), "roto");
    }

    #[test]
    fn an_uncaught_exception_is_an_error_line() {
        let heard = exception_line(&json!({
            "exceptionDetails": {
                "text": "Uncaught",
                "exception": { "type": "object", "description": "TypeError: x is not a function\n    at app.js:3" }
            }
        }));
        assert_eq!(heard, Some(Heard::Said { level: "error".into(), text: "TypeError: x is not a function\n    at app.js:3".into() }));
        assert_eq!(exception_line(&json!({ "exceptionDetails": { "text": "Script error." } })), Some(Heard::Said { level: "error".into(), text: "Script error.".into() }));
    }
}
