use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use sens_agent::canon::checkpoint::Restored;
use sens_agent::canon::circuit::{self, Excepted};
use sens_agent::canon::log::{self, Avoided};
use sens_agent::canon::state;
use sens_agent::canon::view;
use sens_agent::chat::Engine;
use sens_agent::sens_canon::verdict::{Finding, ProjectRules};
use tauri::State;

const MAP_PATIENCE: Duration = Duration::from_secs(30);

#[tauri::command(async)]
pub fn canon_held(work: String) -> Option<Vec<Finding>> {
    circuit::held(Path::new(&work))
}

#[tauri::command(async)]
pub fn canon_accept(work: String) -> Result<(), String> {
    circuit::accept(Path::new(&work))
}

#[tauri::command(async)]
pub fn canon_undo(work: String) -> Result<Restored, String> {
    circuit::undo(Path::new(&work))
}

#[tauri::command(async)]
pub fn canon_fix(work: String) -> Option<String> {
    circuit::fix_request(Path::new(&work))
}

#[tauri::command(async)]
pub fn canon_retry(engine: State<Arc<Engine>>, session_id: String) -> Result<(), String> {
    engine.retry(&session_id)
}

#[tauri::command(async)]
pub fn canon_exceptions(work: String) -> Vec<Excepted> {
    circuit::exceptions(Path::new(&work))
}

#[tauri::command(async)]
pub fn canon_retract(work: String, key: String) -> Result<(), String> {
    circuit::retract(Path::new(&work), &key)
}

#[tauri::command(async)]
pub fn canon_rules(work: String) -> ProjectRules {
    state::rules(Path::new(&work))
}

#[tauri::command(async)]
pub fn canon_set_rules(work: String, rules: ProjectRules) -> Result<(), String> {
    state::save_rules(Path::new(&work), &rules)
}

#[tauri::command(async)]
pub fn canon_avoided(work: String, since: u64) -> Avoided {
    log::avoided(Path::new(&work), since)
}

#[tauri::command(async)]
pub fn canon_map(engine: State<Arc<Engine>>, work: String) -> Option<view::Shown> {
    engine.keeper().ready(Path::new(&work), MAP_PATIENCE).map(|project| view::shown(&project))
}

#[tauri::command(async)]
pub fn canon_reach(engine: State<Arc<Engine>>, work: String, file: String) -> Option<view::Reach> {
    engine.keeper().ready(Path::new(&work), MAP_PATIENCE).and_then(|project| view::reach(&project, &file))
}
