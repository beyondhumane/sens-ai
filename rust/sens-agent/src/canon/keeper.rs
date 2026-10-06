use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use sens_canon::relevant::Catalog;
use sens_index::build;
use sens_index::index::Index;
use sens_index::map::Map;

pub struct Project {
    pub index: Index,
    pub catalog: Catalog,
    pub map: Map,
}

impl Project {
    pub fn of(work: &Path) -> Project {
        let index = build::build(work);
        let catalog = Catalog::of(&index);
        let map = Map::of(&index);
        Project { index, catalog, map }
    }
}

#[derive(Default)]
struct Slot {
    project: Mutex<Option<Arc<Project>>>,
    built: Condvar,
    building: AtomicBool,
    turn: Mutex<()>,
}

#[derive(Default)]
pub struct Keeper {
    slots: Mutex<HashMap<PathBuf, Arc<Slot>>>,
}

impl Keeper {
    fn slot(&self, work: &Path) -> Arc<Slot> {
        let mut slots = self.slots.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        slots.entry(work.to_path_buf()).or_default().clone()
    }

    fn store(slot: &Slot, project: Project) -> Arc<Project> {
        let project = Arc::new(project);
        *slot.project.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(project.clone());
        slot.built.notify_all();
        project
    }

    pub fn warm(&self, work: &Path) {
        let slot = self.slot(work);
        let known = slot.project.lock().map(|project| project.is_some()).unwrap_or(false);
        if known || slot.building.swap(true, Ordering::SeqCst) {
            return;
        }
        let work = work.to_path_buf();
        std::thread::spawn(move || {
            Keeper::store(&slot, Project::of(&work));
            slot.building.store(false, Ordering::SeqCst);
        });
    }

    pub fn ready(&self, work: &Path, patience: Duration) -> Option<Arc<Project>> {
        self.warm(work);
        let slot = self.slot(work);
        let project = slot.project.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let (project, _) = slot.built.wait_timeout_while(project, patience, |project| project.is_none()).unwrap_or_else(|poisoned| poisoned.into_inner());
        project.clone()
    }

    pub fn refresh(&self, work: &Path) -> Arc<Project> {
        Keeper::store(&self.slot(work), Project::of(work))
    }

    pub fn exclusive<T>(&self, work: &Path, run: impl FnOnce() -> T) -> T {
        let slot = self.slot(work);
        let _turn = slot.turn.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join("sens-keeper").join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/a.ts"), "export function one() { return 1; }\n").unwrap();
        root
    }

    #[test]
    fn a_project_is_indexed_once_in_the_background_and_refreshed_on_demand() {
        let root = folder("warm");
        let keeper = Keeper::default();
        keeper.warm(&root);
        let project = keeper.ready(&root, Duration::from_secs(20)).unwrap();
        assert_eq!(project.index.symbols.len(), 1);
        std::fs::write(root.join("src/b.ts"), "export function two() { return 2; }\n").unwrap();
        assert_eq!(keeper.ready(&root, Duration::ZERO).unwrap().index.symbols.len(), 1);
        assert_eq!(keeper.refresh(&root).index.symbols.len(), 2);
        assert_eq!(keeper.ready(&root, Duration::ZERO).unwrap().index.symbols.len(), 2);
    }

    #[test]
    fn work_in_one_folder_is_taken_in_turns() {
        let root = folder("turns");
        let keeper = Arc::new(Keeper::default());
        let order = Arc::new(Mutex::new(Vec::new()));
        let threads: Vec<_> = (0..4)
            .map(|at| {
                let (keeper, order, root) = (keeper.clone(), order.clone(), root.clone());
                std::thread::spawn(move || {
                    keeper.exclusive(&root, || {
                        order.lock().unwrap().push(format!("in{at}"));
                        std::thread::sleep(Duration::from_millis(20));
                        order.lock().unwrap().push(format!("out{at}"));
                    })
                })
            })
            .collect();
        threads.into_iter().for_each(|thread| thread.join().unwrap());
        let order = order.lock().unwrap();
        assert!(order.chunks(2).all(|pair| pair[0].replace("in", "") == pair[1].replace("out", "")), "{order:?}");
    }
}
