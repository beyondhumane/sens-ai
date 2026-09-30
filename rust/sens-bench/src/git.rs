use std::path::Path;
use std::process::Command;

pub fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let done = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=bench@sens.dev", "-c", "user.name=sens-bench", "-c", "core.autocrlf=false"])
        .args(args)
        .output()
        .map_err(|error| format!("git: {error}"))?;
    if !done.status.success() {
        return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&done.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&done.stdout).into_owned())
}

pub fn export(base: &str, work: &Path) -> Result<(), String> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    std::fs::create_dir_all(work).map_err(|error| format!("{}: {error}", work.display()))?;
    let archive = work.with_extension("tar");
    git(&source, &["archive", "--format=tar", "-o", &archive.to_string_lossy(), base])?;
    let unpacked = Command::new("tar").arg("-xf").arg(&archive).arg("-C").arg(work).output().map_err(|error| format!("tar: {error}"))?;
    let _ = std::fs::remove_file(&archive);
    if !unpacked.status.success() {
        return Err(format!("tar: {}", String::from_utf8_lossy(&unpacked.stderr).trim()));
    }
    Ok(())
}

pub fn baseline(dir: &Path) -> Result<(), String> {
    git(dir, &["init", "-q"])?;
    git(dir, &["add", "-A"])?;
    git(dir, &["commit", "-q", "--allow-empty", "-m", "base"])?;
    Ok(())
}

pub struct Changes {
    pub numstat: String,
    pub statuses: String,
    pub patch: String,
}

pub fn changes(dir: &Path) -> Result<Changes, String> {
    git(dir, &["add", "-A"])?;
    Ok(Changes {
        numstat: git(dir, &["diff", "--cached", "--ignore-cr-at-eol", "--numstat", "HEAD"])?,
        statuses: git(dir, &["diff", "--cached", "--name-status", "HEAD"])?,
        patch: git(dir, &["diff", "--cached", "--ignore-cr-at-eol", "HEAD"])?,
    })
}

pub fn at_base(dir: &Path, path: &str) -> String {
    git(dir, &["show", &format!("HEAD:{path}")]).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_commit_of_this_repository_can_be_the_start_of_a_task() {
        let work = std::env::temp_dir().join("sens-bench-export");
        let _ = std::fs::remove_dir_all(&work);
        export("7eb9269", &work).unwrap();
        assert!(work.join("rust/sens-app/ui/src/shared/format.js").is_file());
        assert!(!work.join("rust/sens-canon").exists());
    }

    #[test]
    fn line_endings_alone_do_not_count_as_changes() {
        let dir = std::env::temp_dir().join("sens-bench-eol");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "uno\ndos\n").unwrap();
        baseline(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "uno\r\ndos\r\ntres\r\n").unwrap();
        let changed = changes(&dir).unwrap();
        assert_eq!(changed.numstat.split('\t').take(2).collect::<Vec<_>>(), ["1", "0"]);
        assert_eq!(at_base(&dir, "a.txt"), "uno\ndos\n");
    }
}
