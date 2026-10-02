use std::io::{self, Write};

#[derive(Debug, PartialEq)]
pub struct Summary {
    pub count: usize,
    pub total: f64,
}

impl Summary {
    pub fn of(text: &str) -> Result<Summary, String> {
        let mut summary = Summary {
            count: 0,
            total: 0.0,
        };
        for (at, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let value: f64 = line
                .parse()
                .map_err(|_| format!("line {}: {line} is not a number", at + 1))?;
            summary.count += 1;
            summary.total += value;
        }
        Ok(summary)
    }

    pub fn mean(&self) -> f64 {
        match self.count {
            0 => 0.0,
            count => self.total / count as f64,
        }
    }

    pub fn line(&self) -> String {
        format!(
            "{} líneas · total {} · media {}",
            self.count,
            self.total,
            self.mean()
        )
    }

    pub fn write_csv(&self, out: &mut impl Write) -> io::Result<()> {
        writeln!(out, "count,total,mean")?;
        writeln!(out, "{},{},{}", self.count, self.total, self.mean())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_counted_and_added_skipping_blank_lines() {
        let summary = Summary::of("10\n\n12\n 20 \n").unwrap();
        assert_eq!(
            summary,
            Summary {
                count: 3,
                total: 42.0
            }
        );
        assert_eq!(summary.line(), "3 líneas · total 42 · media 14");
    }

    #[test]
    fn a_line_that_is_not_a_number_is_named() {
        assert_eq!(
            Summary::of("1\nx\n").unwrap_err(),
            "line 2: x is not a number"
        );
    }

    #[test]
    fn the_csv_has_a_header_and_one_row() {
        let mut out = Vec::new();
        Summary::of("2\n4\n").unwrap().write_csv(&mut out).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "count,total,mean\n2,6,3\n");
    }
}
