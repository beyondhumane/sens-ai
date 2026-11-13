use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub struct Front<'t> {
    pub fields: BTreeMap<String, Vec<String>>,
    pub odd: Vec<u32>,
    pub body: &'t str,
    pub body_line: u32,
}

pub const NEVER_CLOSED: &str = "its frontmatter opens with `---` and never closes";

fn values(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    let items: Vec<&str> = match raw.strip_prefix('[').and_then(|inner| inner.strip_suffix(']')) {
        Some(inner) => inner.split(',').collect(),
        None => vec![raw],
    };
    items.into_iter().map(|item| item.trim().trim_matches(['"', '\'']).to_string()).filter(|item| !item.is_empty()).collect()
}

fn field(line: &str) -> Option<(String, Vec<String>)> {
    let (key, value) = line.split_once(':')?;
    let key = key.trim();
    (!key.is_empty() && !key.contains(char::is_whitespace) && !key.starts_with('-')).then(|| (key.to_string(), values(value)))
}

pub fn read(text: &str) -> Result<Front<'_>, &'static str> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut lines = text.split_inclusive('\n');
    let plain = Front { fields: BTreeMap::new(), odd: Vec::new(), body: text, body_line: 1 };
    let Some(first) = lines.next() else { return Ok(plain) };
    if first.trim() != "---" {
        return Ok(plain);
    }
    let mut front = Front { body_line: 2, body: "", ..plain };
    let mut offset = first.len();
    for line in lines {
        offset += line.len();
        front.body_line += 1;
        let said = line.trim();
        if said == "---" {
            front.body = &text[offset..];
            return Ok(front);
        }
        if said.is_empty() {
            continue;
        }
        match field(said) {
            Some((key, values)) => {
                front.fields.insert(key, values);
            }
            None => front.odd.push(front.body_line - 1),
        }
    }
    Err(NEVER_CLOSED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_hold_one_value_or_a_list_and_the_body_starts_after_the_fence() {
        let front = read("---\nstatus: prototype\npillars: [movimiento-preciso, \"riesgo\"]\nempty:\n---\n# Dash\n").unwrap();
        assert_eq!(front.fields["status"], ["prototype"]);
        assert_eq!(front.fields["pillars"], ["movimiento-preciso", "riesgo"]);
        assert!(front.fields["empty"].is_empty());
        assert_eq!((front.body, front.body_line), ("# Dash\n", 6));
        assert!(front.odd.is_empty());
    }

    #[test]
    fn a_line_that_is_not_key_value_is_reported_by_its_line() {
        let front = read("---\npillars:\n  - movimiento\n---\nbody\n").unwrap();
        assert_eq!(front.odd, [3]);
        assert!(front.fields["pillars"].is_empty());
    }

    #[test]
    fn without_frontmatter_the_whole_text_is_body_and_an_open_one_is_refused() {
        let plain = read("# Title\n").unwrap();
        assert_eq!((plain.body, plain.body_line, plain.fields.len()), ("# Title\n", 1, 0));
        assert_eq!(read("\u{feff}---\r\nstatus: idea\r\n---\r\nx\r\n").unwrap().fields["status"], ["idea"]);
        assert_eq!(read("---\nstatus: idea\n# Dash\n"), Err(NEVER_CLOSED));
    }
}
