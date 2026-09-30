use std::path::{Path, PathBuf};
use std::process::Command;

use sens_agent::canon::review::{Haiku, Reviewer};
use sens_canon::relevant::Catalog;
use sens_canon::review::Review;

fn git(dir: &Path, args: &[&str]) -> String {
    let done = Command::new("git").arg("-C").arg(dir).args(args).output().expect("git");
    String::from_utf8_lossy(&done.stdout).into_owned()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [run, prompt] = args.as_slice() else {
        eprintln!("review_run <carpeta de la ejecución> <prompt.md>");
        std::process::exit(2);
    };
    let run = PathBuf::from(run);
    let base = std::env::temp_dir().join("sens-review-run").join(run.file_name().unwrap_or_default());
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("carpeta");
    let archive = base.with_extension("tar");
    git(&run, &["archive", "--format=tar", "-o", &archive.to_string_lossy(), "HEAD"]);
    Command::new("tar").arg("-xf").arg(&archive).arg("-C").arg(&base).status().expect("tar");
    let index = sens_index::build::build(&run);
    let catalog = Catalog::of(&index);
    let diff = git(&run, &["diff", "--no-color", "--no-renames", "-U3", "HEAD", "--", ".", ":(exclude).sens", ":(exclude)**/_accept/**", ":(exclude)accept"]);
    let request = std::fs::read_to_string(prompt).expect("prompt");
    let review = Review::of(&index, &catalog, &request, &diff);
    for (file, found) in &review.candidates {
        println!("candidatos {file}: {}", found.iter().map(|candidate| format!("{} ({}:{})", candidate.name, candidate.file, candidate.line)).collect::<Vec<_>>().join(", "));
    }
    let answer = Haiku::default().review(&review.prompt()).expect("revisor").answer;
    println!("respuesta: {answer}");
    for finding in review.findings(&answer) {
        println!("{:?} {:?} {}:{} {}", finding.rule, finding.severity, finding.file, finding.line, finding.message);
    }
}
