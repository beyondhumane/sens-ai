use std::path::PathBuf;

use crate::build;
use crate::index::Index;

pub fn written(group: &str, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(group).join(name);
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    for (path, content) in files {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    root
}

pub fn indexed(group: &str, name: &str, files: &[(&str, &str)]) -> Index {
    build::build(&written(group, name, files))
}
