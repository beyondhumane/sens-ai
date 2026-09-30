use std::path::Path;
use std::process::Command;

const BUILD_ELSEWHERE: &str = "CARGO_TARGET_DIR";

pub struct Outcome {
    pub ok: bool,
    pub output: String,
}

pub fn run(dir: &Path, line: &str) -> Outcome {
    if line.trim().is_empty() {
        return Outcome { ok: true, output: String::new() };
    }
    match shell(line).current_dir(dir).env_remove(BUILD_ELSEWHERE).output() {
        Ok(done) => Outcome {
            ok: done.status.success(),
            output: format!("{}{}", String::from_utf8_lossy(&done.stdout), String::from_utf8_lossy(&done.stderr)),
        },
        Err(error) => Outcome { ok: false, output: error.to_string() },
    }
}

#[cfg(windows)]
fn shell(line: &str) -> Command {
    use std::os::windows::process::CommandExt;
    let mut command = Command::new("cmd");
    command.arg("/C").raw_arg(line);
    command
}

#[cfg(not(windows))]
fn shell(line: &str) -> Command {
    let mut command = Command::new("sh");
    command.arg("-c").arg(line);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_runs_in_the_folder_and_says_whether_it_worked() {
        let here = std::env::temp_dir();
        assert!(run(&here, "node -e \"process.exit(0)\"").ok);
        let failed = run(&here, "node -e \"console.log('mal'); process.exit(3)\"");
        assert!(!failed.ok);
        assert!(failed.output.contains("mal"));
    }

    #[test]
    fn a_task_builds_in_its_own_folder_whatever_the_bench_was_built_with() {
        let here = std::env::temp_dir();
        let seen = run(&here, "node -e \"process.exit(process.env.CARGO_TARGET_DIR ? 1 : 0)\"");
        assert!(seen.ok, "{}", seen.output);
    }

    #[test]
    fn nothing_to_run_is_a_success() {
        assert!(run(&std::env::temp_dir(), "  ").ok);
    }
}
