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
        numstat: git(dir, &["diff", "--cached", "--numstat", "HEAD"])?,
        statuses: git(dir, &["diff", "--cached", "--name-status", "HEAD"])?,
        patch: git(dir, &["diff", "--cached", "HEAD"])?,
    })
}

pub fn at_base(dir: &Path, path: &str) -> String {
    git(dir, &["show", &format!("HEAD:{path}")]).unwrap_or_default()
}
