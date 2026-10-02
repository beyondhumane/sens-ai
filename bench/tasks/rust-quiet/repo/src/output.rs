use std::fs::File;
use std::path::{Component, Path};

pub fn open_output(path: &Path) -> Result<File, String> {
    let escapes = path.components().any(|part| {
        matches!(
            part,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    });
    if escapes {
        return Err(format!(
            "{} must stay inside the working folder",
            path.display()
        ));
    }
    File::create(path).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_output_outside_the_working_folder_is_refused() {
        assert!(open_output(Path::new("../escape.csv")).is_err());
        assert!(open_output(Path::new("out/../../escape.csv")).is_err());
        assert!(open_output(&std::env::temp_dir().join("escape.csv")).is_err());
    }
}
