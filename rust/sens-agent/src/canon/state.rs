use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use sens_canon::verdict::{Exceptions, Finding, ProjectRules};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::checkpoint::Tree;

const FOLDER: [&str; 2] = [".sens", "canon"];

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct State {
    pub approved: Option<Tree>,
    pub seen: Option<Tree>,
    pub end: Option<Tree>,
    pub turn: u64,
    pub step: u64,
    pub rounds: u32,
    pub helper_rounds: BTreeMap<String, u32>,
    pub held: Option<Vec<Finding>>,
    pub dead: Vec<(String, String)>,
    pub dead_known: bool,
    pub request: String,
    pub canon: BTreeMap<String, String>,
}

fn file(work: &Path, name: &str) -> PathBuf {
    FOLDER.iter().fold(work.to_path_buf(), |path, part| path.join(part)).join(name)
}

fn load<T: DeserializeOwned + Default>(work: &Path, name: &str) -> T {
    std::fs::read_to_string(file(work, name)).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

fn save<T: Serialize>(work: &Path, name: &str, value: &T) -> Result<(), String> {
    let target = file(work, name);
    let folder = target.parent().unwrap_or(work);
    std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    let written = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    let staged = target.with_extension("json.new");
    std::fs::write(&staged, written).map_err(|error| format!("{}: {error}", staged.display()))?;
    std::fs::rename(&staged, &target).map_err(|error| format!("{}: {error}", target.display()))
}

impl State {
    pub fn load(work: &Path) -> State {
        load(work, "state.json")
    }

    pub fn save(&self, work: &Path) -> Result<(), String> {
        save(work, "state.json", self)
    }
}

pub fn exceptions(work: &Path) -> Exceptions {
    load(work, "exceptions.json")
}

pub fn save_exceptions(work: &Path, exceptions: &Exceptions) -> Result<(), String> {
    save(work, "exceptions.json", exceptions)
}

pub fn considered(work: &Path) -> BTreeSet<String> {
    load(work, "considered.json")
}

pub fn save_considered(work: &Path, keys: &BTreeSet<String>) -> Result<(), String> {
    save(work, "considered.json", keys)
}

pub fn rules(work: &Path) -> ProjectRules {
    load(work, "rules.json")
}

pub fn save_rules(work: &Path, rules: &ProjectRules) -> Result<(), String> {
    save(work, "rules.json", rules)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join("sens-state").join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn the_state_the_exceptions_and_the_rules_survive_a_restart() {
        let root = folder("round-trip");
        assert_eq!(State::load(&root), State::default());
        let state = State { approved: Some(Tree("abc".into())), turn: 3, request: "hola".into(), ..State::default() };
        state.save(&root).unwrap();
        assert_eq!(State::load(&root), state);
        save_exceptions(&root, &Exceptions { keys: ["R3:dayjs".to_string()].into() }).unwrap();
        assert!(exceptions(&root).keys.contains("R3:dayjs"));
        save_rules(&root, &ProjectRules { no_comments: true }).unwrap();
        assert!(rules(&root).no_comments);
        assert!(std::fs::read_to_string(root.join(".sens/canon/rules.json")).unwrap().contains("noComments"));
    }

    #[test]
    fn a_damaged_state_reads_as_a_fresh_one() {
        let root = folder("damaged");
        std::fs::create_dir_all(root.join(".sens/canon")).unwrap();
        std::fs::write(root.join(".sens/canon/state.json"), "{ not json").unwrap();
        assert_eq!(State::load(&root), State::default());
    }
}
