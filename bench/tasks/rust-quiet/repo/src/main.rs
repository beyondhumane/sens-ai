mod config;
mod output;
mod summary;

use std::process::ExitCode;

use summary::Summary;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("report: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let config = config::parse(args)?;
    let text = std::fs::read_to_string(&config.input)
        .map_err(|error| format!("{}: {error}", config.input.display()))?;
    let summary = Summary::of(&text)?;
    if let Some(path) = &config.output {
        let mut file = output::open_output(path)?;
        summary
            .write_csv(&mut file)
            .map_err(|error| format!("{}: {error}", path.display()))?;
    }
    println!("{}", summary.line());
    Ok(())
}
