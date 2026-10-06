mod console;
mod repo;
mod sessions;
mod surface;
mod web;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use sens_agent::canon::keeper::Keeper;
use sens_agent::canon::tools as index;
use serde_json::{Value, json};

pub use crate::browser::Request;
use crate::terminal::Consoles;

pub const SERVER: &str = "sens";
pub const REPLACED: &str = "Bash,PowerShell,Monitor";
const INDEX_PATIENCE: Duration = Duration::from_secs(20);
pub const QUICK: Duration = Duration::from_secs(5);
const UNINDEXED: &str = "Sens has not finished indexing this project yet; try again in a moment.";

#[derive(Clone, Debug, PartialEq)]
pub struct Scope {
    pub session: String,
    pub within: Vec<String>,
}

impl Scope {
    pub fn root(&self) -> &Path {
        Path::new(self.within.first().map_or("", String::as_str))
    }

    pub fn work(&self) -> &Path {
        Path::new(self.within.last().map_or("", String::as_str))
    }

    pub fn holds(&self, path: &Path) -> bool {
        let path = folded(&path.to_string_lossy());
        self.within.iter().map(|folder| folded(folder)).any(|folder| path == folder || path.starts_with(&format!("{folder}/")))
    }
}

fn folded(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

pub trait Ui: Sync {
    fn act_within(&self, act: &str, input: Value, patience: Duration) -> Result<String, String>;

    fn act(&self, act: &str, input: Value) -> Result<String, String> {
        self.act_within(act, input, QUICK)
    }
}

pub struct Picture {
    pub media_type: String,
    pub data: String,
}

pub trait Sens: Sync {
    fn screenshot(&self) -> Option<Picture>;
    fn notify(&self, title: &str, body: &str) -> Result<(), String>;
    fn devtools(&self, method: &str, params: Value) -> Result<Value, String>;
    fn console(&self) -> Vec<(String, String)>;
    fn requests(&self) -> Vec<Request>;
    fn data(&self) -> Option<PathBuf>;
    fn busy(&self, session: &str) -> bool;
}

#[derive(Debug, PartialEq)]
pub enum Said {
    Text(String),
    Picture { media_type: String, data: String, caption: String },
}

impl From<String> for Said {
    fn from(text: String) -> Said {
        Said::Text(text)
    }
}

impl From<&str> for Said {
    fn from(text: &str) -> Said {
        Said::Text(text.to_string())
    }
}

impl Said {
    pub fn content(&self) -> Value {
        match self {
            Said::Text(text) => json!([{ "type": "text", "text": text }]),
            Said::Picture { media_type, data, caption } => json!([
                { "type": "image", "data": data, "mimeType": media_type },
                { "type": "text", "text": caption }
            ]),
        }
    }
}

pub type Tell = Arc<dyn Fn(&str, Value) + Send + Sync>;

pub struct Desk<'a> {
    pub consoles: &'a Consoles,
    pub keeper: &'a Keeper,
    pub ui: &'a dyn Ui,
    pub sens: &'a dyn Sens,
    pub tell: Tell,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Level {
    Read,
    Show,
    Change,
}

pub struct Tool {
    pub name: &'static str,
    pub title: &'static str,
    pub level: Level,
    pub description: &'static str,
    pub input: fn() -> Value,
    pub run: fn(&Desk, &Scope, &Value) -> Result<Said, String>,
}

impl Tool {
    fn spec(&self) -> Value {
        let input = (self.input)();
        json!({
            "name": self.name,
            "title": self.title,
            "description": self.description,
            "inputSchema": {
                "type": "object",
                "properties": input["properties"].clone(),
                "required": input["required"].clone(),
                "additionalProperties": false
            },
            "annotations": {
                "readOnlyHint": self.level == Level::Read,
                "destructiveHint": self.level == Level::Change,
                "openWorldHint": false
            }
        })
    }
}

fn domains() -> impl Iterator<Item = &'static Tool> {
    console::TOOLS.iter().chain(surface::TOOLS).chain(web::TOOLS).chain(sessions::TOOLS).chain(repo::TOOLS)
}

pub fn listed() -> Vec<Value> {
    domains().map(Tool::spec).chain(index::listed()).collect()
}

pub fn allowed() -> String {
    domains()
        .filter(|tool| tool.level != Level::Change)
        .map(|tool| tool.name)
        .chain(index::NAMES)
        .map(|name| format!("mcp__{SERVER}__{name}"))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn call(desk: &Desk, scope: &Scope, name: &str, arguments: &Value) -> Option<Result<Said, String>> {
    if let Some(tool) = domains().find(|tool| tool.name == name) {
        return Some((tool.run)(desk, scope, arguments));
    }
    if !index::NAMES.contains(&name) {
        return None;
    }
    let answer = desk.keeper.ready(scope.work(), INDEX_PATIENCE).and_then(|project| index::call(&project, name, arguments));
    Some(answer.unwrap_or_else(|| Err(UNINDEXED.to_string())).map(Said::from))
}

pub fn text(arguments: &Value, key: &str) -> Result<String, String> {
    arguments[key].as_str().map(str::trim).filter(|text| !text.is_empty()).map(str::to_string).ok_or_else(|| format!("`{key}` is missing"))
}

pub fn number(arguments: &Value, key: &str) -> Result<u64, String> {
    arguments[key].as_u64().ok_or_else(|| format!("`{key}` is missing"))
}

pub fn schema(properties: Value, required: &[&str]) -> Value {
    json!({ "properties": properties, "required": required })
}

#[cfg(test)]
pub mod testing {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    pub struct Window {
        pub asked: Mutex<Vec<(String, Value)>>,
        pub told: Arc<Mutex<Vec<(String, Value)>>>,
        pub notified: Mutex<Vec<(String, String)>>,
        pub hidden: bool,
        pub called: Mutex<Vec<(String, Value)>>,
        pub answers: Mutex<Vec<(String, Result<Value, String>)>>,
        pub said: Vec<(String, String)>,
        pub requested: Vec<Request>,
        pub base: Option<PathBuf>,
        pub working: Vec<String>,
    }

    impl Ui for Window {
        fn act_within(&self, act: &str, input: Value, _: std::time::Duration) -> Result<String, String> {
            self.asked.lock().unwrap().push((act.to_string(), input.clone()));
            Ok(format!("{act} {input}"))
        }
    }

    impl Sens for Window {
        fn screenshot(&self) -> Option<Picture> {
            (!self.hidden).then(|| Picture { media_type: "image/png".into(), data: "iVBORw0KGgo=".into() })
        }

        fn notify(&self, title: &str, body: &str) -> Result<(), String> {
            self.notified.lock().unwrap().push((title.to_string(), body.to_string()));
            Ok(())
        }

        fn devtools(&self, method: &str, params: Value) -> Result<Value, String> {
            self.called.lock().unwrap().push((method.to_string(), params.clone()));
            let answer = self.answers.lock().unwrap().iter().find(|(asked, _)| method.starts_with(asked.as_str()) || params.to_string().contains(asked.as_str())).map(|(_, answer)| answer.clone());
            answer.unwrap_or_else(|| Ok(json!({})))
        }

        fn console(&self) -> Vec<(String, String)> {
            self.said.clone()
        }

        fn requests(&self) -> Vec<Request> {
            self.requested.clone()
        }

        fn data(&self) -> Option<PathBuf> {
            self.base.clone()
        }

        fn busy(&self, session: &str) -> bool {
            self.working.iter().any(|one| one == session)
        }
    }

    impl Window {
        pub fn desk<'a>(&'a self, consoles: &'a Consoles, keeper: &'a Keeper) -> Desk<'a> {
            let told = self.told.clone();
            Desk { consoles, keeper, ui: self, sens: self, tell: Arc::new(move |event, payload| told.lock().unwrap().push((event.to_string(), payload))) }
        }
    }

    pub fn scope(work: &Path) -> Scope {
        Scope { session: "s1".into(), within: vec![work.to_string_lossy().into_owned()] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_tool_is_listed_once_and_only_the_changing_ones_wait_for_permission() {
        let names: Vec<String> = listed().iter().map(|tool| tool["name"].as_str().unwrap().to_string()).collect();
        assert_eq!(names.iter().collect::<HashSet<_>>().len(), names.len());
        let allowed: Vec<&str> = allowed().leak().split(',').collect();
        for tool in domains() {
            let name = format!("mcp__{SERVER}__{}", tool.name);
            assert_eq!(allowed.contains(&name.as_str()), tool.level != Level::Change, "{name}");
        }
        assert!(index::NAMES.iter().all(|name| allowed.contains(&format!("mcp__{SERVER}__{name}").as_str())));
        assert!(listed().iter().all(|tool| tool["inputSchema"]["additionalProperties"] == false));
    }

    #[test]
    fn a_scope_holds_its_folders_whatever_the_slashes_and_case() {
        let scope = Scope { session: "s".into(), within: vec!["C:\\Demo".into(), "C:/Demo/.sens/worktrees/ab".into()] };
        assert_eq!(scope.work(), Path::new("C:/Demo/.sens/worktrees/ab"));
        assert!(scope.holds(Path::new("c:/demo/src")));
        assert!(scope.holds(Path::new("C:\\Demo")));
        assert!(!scope.holds(Path::new("C:/Demolition")));
        assert!(!scope.holds(Path::new("D:/elsewhere")));
    }

    #[test]
    fn an_unknown_tool_is_not_answered_and_an_unready_index_says_so() {
        let consoles = Consoles::default();
        let keeper = Keeper::default();
        let window = testing::Window::default();
        let desk = window.desk(&consoles, &keeper);
        let scope = testing::scope(Path::new("Z:/sens/no-such-project"));
        assert!(call(&desk, &scope, "rm", &json!({})).is_none());
        assert!(text(&json!({ "a": "  " }), "a").is_err());
        assert_eq!(number(&json!({ "a": 3 }), "a"), Ok(3));
    }

    #[test]
    fn a_picture_travels_as_an_image_with_its_caption() {
        let picture = Said::Picture { media_type: "image/png".into(), data: "AA==".into(), caption: "Sens".into() };
        assert_eq!(picture.content(), json!([{ "type": "image", "data": "AA==", "mimeType": "image/png" }, { "type": "text", "text": "Sens" }]));
        assert_eq!(Said::from("hola").content(), json!([{ "type": "text", "text": "hola" }]));
    }
}
