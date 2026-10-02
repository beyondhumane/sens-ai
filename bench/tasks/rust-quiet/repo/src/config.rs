use std::path::PathBuf;

const USAGE: &str = "usage: report <input> [--output <file.csv>]";

#[derive(Debug, Default, PartialEq)]
pub struct Config {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub quiet: bool,
}

pub fn parse(args: &[String]) -> Result<Config, String> {
    let mut config = Config::default();
    let mut input = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--output" => {
                let path = rest.next().ok_or("--output needs a path")?;
                config.output = Some(PathBuf::from(path));
            }
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            path => input = Some(PathBuf::from(path)),
        }
    }
    config.input = input.ok_or(USAGE)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn the_input_and_the_output_are_read_in_any_order() {
        let config = parse(&args(&["--output", "out.csv", "data.txt"])).unwrap();
        assert_eq!(config.input, PathBuf::from("data.txt"));
        assert_eq!(config.output, Some(PathBuf::from("out.csv")));
    }

    #[test]
    fn unknown_options_and_a_missing_input_are_refused() {
        assert!(parse(&args(&["data.txt", "--verbose"])).is_err());
        assert!(parse(&args(&["--output", "out.csv"])).is_err());
        assert!(parse(&args(&["data.txt", "--output"])).is_err());
    }
}
