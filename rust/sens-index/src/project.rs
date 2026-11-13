use std::path::Path;

const UNITY_GENERATED: [&str; 9] = ["library", "temp", "obj", "logs", "usersettings", "build", "builds", "memorycaptures", "recordings"];

pub fn unity_version(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join("ProjectSettings").join("ProjectVersion.txt")).ok()?;
    let version = text.lines().find_map(|line| line.strip_prefix("m_EditorVersion:"))?.trim();
    (!version.is_empty()).then(|| version.to_string())
}

pub fn is_unity(root: &Path) -> bool {
    unity_version(root).is_some()
}

pub fn addressables(root: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(root.join("Packages").join("manifest.json")) else { return false };
    serde_json::from_str::<serde_json::Value>(&text).is_ok_and(|manifest| manifest["dependencies"].get("com.unity.addressables").is_some())
}

fn generated(root: &Path) -> &'static [&'static str] {
    if is_unity(root) { &UNITY_GENERATED } else { &[] }
}

pub fn skipping(root: &Path, base: &'static [&'static str]) -> impl Fn(&ignore::DirEntry) -> bool + Send + Sync + 'static {
    let generated = generated(root);
    move |entry| {
        if entry.depth() == 0 {
            return true;
        }
        let name = entry.file_name().to_str().unwrap_or("");
        !base.contains(&name) && !generated.contains(&name.to_ascii_lowercase().as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
        crate::fixture::written("sens-index-project", name, files)
    }

    const VERSION: (&str, &str) = ("ProjectSettings/ProjectVersion.txt", "m_EditorVersion: 6000.0.23f1\nm_EditorVersionWithRevision: 6000.0.23f1 (abc123)\n");

    fn indexed(root: &Path) -> Vec<String> {
        crate::build::collect(root)
            .iter()
            .map(|path| path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"))
            .collect()
    }

    #[test]
    fn the_editor_version_is_the_detection_and_a_project_without_it_is_not_unity() {
        let unity = project("version", &[VERSION]);
        assert_eq!(unity_version(&unity).as_deref(), Some("6000.0.23f1"));
        assert!(is_unity(&unity));
        let plain = project("no-version", &[("package.json", "{}")]);
        assert_eq!(unity_version(&plain), None);
        assert!(!is_unity(&plain));
        let empty = project("empty-version", &[("ProjectSettings/ProjectVersion.txt", "m_EditorVersion:\n")]);
        assert!(!is_unity(&empty));
    }

    #[test]
    fn addressables_count_only_when_the_manifest_depends_on_them() {
        let with = project("addressables", &[VERSION, ("Packages/manifest.json", r#"{ "dependencies": { "com.unity.addressables": "1.21.19", "com.unity.ugui": "1.0.0" } }"#)]);
        assert!(addressables(&with));
        let named_elsewhere = project("addressables-elsewhere", &[VERSION, ("Packages/manifest.json", r#"{ "dependencies": { "com.unity.ugui": "1.0.0" }, "testables": ["com.unity.addressables"] }"#)]);
        assert!(!addressables(&named_elsewhere));
        assert!(!addressables(&project("addressables-none", &[VERSION])));
    }

    #[test]
    fn a_unity_project_hides_what_the_editor_generates_and_keeps_what_the_person_wrote() {
        let root = project(
            "generated",
            &[
                VERSION,
                ("Assets/Scripts/Player.cs", "class Player {}"),
                ("Packages/manifest.json", "{}"),
                ("Library/PackageCache/com.unity.ugui/Runtime/Text.cs", "class Text {}"),
                ("Temp/Generated.cs", "class Generated {}"),
                ("obj/Debug/Rebuild.cs", "class Rebuild {}"),
                ("Logs/worker.cs", "class Worker {}"),
                ("Builds/Windows/Shipped.cs", "class Shipped {}"),
            ],
        );
        assert_eq!(indexed(&root), ["Assets/Scripts/Player.cs"]);
    }

    #[test]
    fn a_project_that_is_not_unity_keeps_its_own_library_folder() {
        let root = project("library", &[("package.json", "{}"), ("Library/helper.ts", "export const helper = () => 1;"), ("node_modules/left-pad/index.js", "module.exports = 1;")]);
        assert_eq!(indexed(&root), ["Library/helper.ts"]);
    }

    #[test]
    fn the_root_is_never_skipped_for_being_named_like_a_folder_we_ignore() {
        let root = project("Builds", &[VERSION, ("Assets/Scripts/Player.cs", "class Player {}")]);
        assert_eq!(indexed(&root), ["Assets/Scripts/Player.cs"]);
    }
}
