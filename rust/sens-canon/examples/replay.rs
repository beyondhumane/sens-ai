use std::path::{Path, PathBuf};
use std::process::Command;

use sens_canon::judge::judge_turn;
use sens_canon::orphans;
use sens_canon::verdict::{Change, Exceptions, ProjectRules};

fn git(dir: &Path, args: &[&str]) -> String {
    let done = Command::new("git").arg("-C").arg(dir).args(args).output().expect("git");
    String::from_utf8_lossy(&done.stdout).into_owned()
}

fn main() {
    let run = PathBuf::from(std::env::args().nth(1).expect("carpeta de la ejecución"));
    let base = std::env::temp_dir().join("sens-replay").join(run.file_name().unwrap_or_default());
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("carpeta");
    let archive = base.with_extension("tar");
    git(&run, &["archive", "--format=tar", "-o", &archive.to_string_lossy(), "HEAD"]);
    Command::new("tar").arg("-xf").arg(&archive).arg("-C").arg(&base).status().expect("tar");
    let index = sens_index::build::build(&base);
    let dead = orphans::dead(&index);
    let mut changes = Vec::new();
    for line in git(&run, &["status", "--porcelain", "-uall"]).lines() {
        let path = line[3..].trim().trim_matches('"').to_string();
        if path.starts_with(".sens/") || path.contains("_accept/") || path.starts_with("accept/") {
            continue;
        }
        let before = Some(git(&run, &["show", &format!("HEAD:{path}")])).filter(|text| !text.is_empty());
        let after = std::fs::read_to_string(run.join(&path)).ok();
        changes.push(Change { path, before, after });
    }
    let (verdict, _) = judge_turn(&index, &changes, &dead, None, &ProjectRules::default(), &Exceptions::default());
    println!("{}: {} cambios", run.file_name().unwrap_or_default().to_string_lossy(), changes.len());
    for finding in verdict.findings {
        println!("  {:?} {:?} {}:{} {}", finding.rule, finding.severity, finding.file, finding.line, finding.message.chars().take(200).collect::<String>());
    }
}
