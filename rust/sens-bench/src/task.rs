use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct Declared {
    language: String,
    #[serde(default)]
    setup: String,
    accept: String,
    #[serde(default)]
    check: String,
    #[serde(default)]
    format: String,
    #[serde(default)]
    reuse: Vec<String>,
    #[serde(default)]
    allow: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub dir: PathBuf,
    pub prompt: String,
    pub language: String,
    pub setup: String,
    pub accept: String,
    pub check: String,
    pub format: String,
    pub reuse: Vec<String>,
    pub allow: Vec<String>,
}

impl Task {
    pub fn repo(&self) -> PathBuf {
        self.dir.join("repo")
    }

    pub fn acceptance(&self) -> PathBuf {
        self.dir.join("accept")
    }
}

pub fn load(dir: &Path) -> Result<Task, String> {
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).map_err(|error| format!("{}: {error}", dir.join(name).display()));
    let declared: Declared = toml::from_str(&read("task.toml")?).map_err(|error| format!("{}: {error}", dir.join("task.toml").display()))?;
    let id = dir.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    Ok(Task {
        id,
        dir: dir.to_path_buf(),
        prompt: read("prompt.md")?.trim().to_string(),
        language: declared.language,
        setup: declared.setup,
        accept: declared.accept,
        check: declared.check,
        format: declared.format,
        reuse: declared.reuse,
        allow: declared.allow,
    })
}

pub fn all(tasks: &Path, only: Option<&str>) -> Result<Vec<Task>, String> {
    let entries = std::fs::read_dir(tasks).map_err(|error| format!("{}: {error}", tasks.display()))?;
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|dir| dir.join("task.toml").is_file())
        .filter(|dir| only.is_none_or(|wanted| dir.file_name().is_some_and(|name| name == wanted)))
        .collect();
    dirs.sort();
    dirs.iter().map(|dir| load(dir)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(name: &str, toml: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("sens-bench-task").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("task.toml"), toml).unwrap();
        std::fs::write(dir.join("prompt.md"), "  Haz algo.\n").unwrap();
        dir
    }

    #[test]
    fn a_task_reads_its_declaration_and_prompt() {
        let dir = written("uno", "language = \"rust\"\naccept = \"cargo test\"\nreuse = [\"quiet\"]\n");
        let task = load(&dir).unwrap();
        assert_eq!(task.id, "uno");
        assert_eq!(task.prompt, "Haz algo.");
        assert_eq!(task.reuse, ["quiet"]);
        assert!(task.setup.is_empty() && task.allow.is_empty());
        assert_eq!(task.repo(), dir.join("repo"));
    }

    #[test]
    fn a_task_without_acceptance_is_refused() {
        let dir = written("sin-aceptar", "language = \"rust\"\n");
        assert!(load(&dir).unwrap_err().contains("accept"));
    }
}
