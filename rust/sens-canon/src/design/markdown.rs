#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub level: usize,
    pub title: String,
    pub line: u32,
    pub body: String,
}

fn heading(line: &str) -> Option<(usize, &str)> {
    let level = line.chars().take_while(|&c| c == '#').count();
    let rest = &line[level..];
    ((1..=6).contains(&level) && (rest.is_empty() || rest.starts_with(' '))).then(|| (level, rest.trim().trim_end_matches('#').trim()))
}

fn fence(line: &str) -> bool {
    let line = line.trim_start();
    line.starts_with("```") || line.starts_with("~~~")
}

pub fn blocks(text: &str, first_line: u32) -> Vec<Block> {
    let mut found = vec![Block { level: 0, title: String::new(), line: first_line, body: String::new() }];
    let mut fenced = false;
    for (at, line) in text.lines().enumerate() {
        fenced ^= fence(line);
        match heading(line).filter(|_| !fenced) {
            Some((level, title)) => found.push(Block { level, title: title.to_string(), line: first_line + at as u32, body: String::new() }),
            None => {
                let open = found.last_mut().expect("there is always a block");
                open.body.push_str(line);
                open.body.push('\n');
            }
        }
    }
    found.retain(|block| block.level > 0 || !block.body.trim().is_empty());
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_open_blocks_with_their_line_and_code_fences_hide_them() {
        let text = "intro\n\n# Dash\nA short burst.\n\n## Numbers ##\n```\n# not a heading\n```\n#hashtag stays text\n";
        let found = blocks(text, 5);
        let shape: Vec<(usize, &str, u32)> = found.iter().map(|block| (block.level, block.title.as_str(), block.line)).collect();
        assert_eq!(shape, [(0, "", 5), (1, "Dash", 7), (2, "Numbers", 10)]);
        assert!(found[2].body.contains("# not a heading") && found[2].body.contains("#hashtag stays text"));
    }

    #[test]
    fn an_empty_preamble_is_dropped() {
        assert_eq!(blocks("\n# Title\nbody\n", 1).len(), 1);
        assert!(blocks("", 1).is_empty());
    }
}
