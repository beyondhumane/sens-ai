const RESERVED: [&str; 46] = [
    "await", "break", "case", "catch", "class", "const", "continue", "debugger", "default", "delete", "do", "else", "enum", "export", "extends", "false", "finally", "for", "function", "if",
    "implements", "import", "in", "instanceof", "interface", "let", "new", "null", "package", "private", "protected", "public", "return", "static", "super", "switch", "this", "throw", "true", "try",
    "typeof", "var", "void", "while", "with", "yield",
];

fn find_from(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    haystack.get(from..)?.find(needle).map(|at| at + from)
}

fn blocks(lower: &str, tag: &str) -> Vec<(usize, usize, usize, usize)> {
    let (open, close) = (format!("<{tag}"), format!("</{tag}"));
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(start) = find_from(lower, &open, from) {
        let Some(opened) = find_from(lower, ">", start) else {
            break;
        };
        let Some(closing) = find_from(lower, &close, opened) else {
            break;
        };
        let end = find_from(lower, ">", closing).map_or(lower.len(), |at| at + 1);
        found.push((start, opened + 1, closing, end));
        from = end;
    }
    found
}

fn word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

fn names_of(bytes: &[u8], from: usize, to: usize, out: &mut [u8]) {
    let mut at = from;
    while at < to {
        if !word_byte(bytes[at]) {
            at += 1;
            continue;
        }
        let start = at;
        while at < to && word_byte(bytes[at]) {
            at += 1;
        }
        let word = std::str::from_utf8(&bytes[start..at]).unwrap_or_default();
        if bytes[start].is_ascii_digit() || RESERVED.contains(&word) {
            continue;
        }
        out[start..at].copy_from_slice(&bytes[start..at]);
        if at < to && !matches!(bytes[at], b'\n' | b'\r') {
            out[at] = b';';
        }
    }
}

pub fn scripted(source: &str) -> String {
    let bytes = source.as_bytes();
    let lower = source.to_ascii_lowercase();
    let mut out: Vec<u8> = bytes.iter().map(|&byte| if matches!(byte, b'\n' | b'\r') { byte } else { b' ' }).collect();
    let scripts = blocks(&lower, "script");
    let mut hidden: Vec<(usize, usize)> = scripts.iter().map(|&(start, _, _, end)| (start, end)).collect();
    hidden.extend(blocks(&lower, "style").into_iter().map(|(start, _, _, end)| (start, end)));
    let mut from = 0;
    while let Some(start) = find_from(&lower, "<!--", from) {
        let end = find_from(&lower, "-->", start).map_or(lower.len(), |at| at + 3);
        hidden.push((start, end));
        from = end;
    }
    hidden.sort_unstable();
    for &(_, body, closing, _) in &scripts {
        out[body..closing].copy_from_slice(&bytes[body..closing]);
    }
    let mut at = 0;
    for (start, end) in hidden {
        if start > at {
            names_of(bytes, at, start, &mut out);
        }
        at = at.max(end);
    }
    names_of(bytes, at, bytes.len(), &mut out);
    String::from_utf8(out).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const VUE: &str = "<template>\n  <button class=\"big\" @click=\"onClick\">{{ formatDate(día) }}</button>\n  <!-- old: legacyThing -->\n</template>\n\n<script setup lang=\"ts\">\nimport { formatDate } from './dates';\nfunction onClick() {}\n</script>\n\n<style scoped>\n.big { color: red }\n</style>\n";

    #[test]
    fn a_component_keeps_its_script_and_every_byte_and_line_where_it_was() {
        let masked = scripted(VUE);
        assert_eq!(masked.len(), VUE.len());
        assert_eq!(masked.lines().count(), VUE.lines().count());
        assert!(masked.contains("import { formatDate } from './dates';\nfunction onClick() {}\n"));
        let script = VUE.find("import").unwrap();
        assert_eq!(&masked[script..script + 6], "import");
    }

    #[test]
    fn the_template_leaves_only_the_names_it_uses_as_statements() {
        let masked = scripted(VUE);
        let template: String = masked.lines().nth(1).unwrap().split_whitespace().collect();
        assert_eq!(template, "button;big;click;onClick;formatDate;d;a;button;");
        assert!(!masked.contains("legacyThing"));
        assert!(!masked.contains("class"));
        assert!(!masked.contains("color"));
        assert!(!masked.contains("scoped"));
    }

    #[test]
    fn a_svelte_template_drops_keywords_and_numbers() {
        let masked = scripted("<script>\nlet count = 0;\n</script>\n{#if count > 10}<p on:click={increment}>{count}px</p>{/if}\n");
        let template: String = masked.lines().nth(3).unwrap().split_whitespace().collect();
        assert_eq!(template, "count;p;on;click;increment;count;px;p;");
    }

    #[test]
    fn a_file_without_markup_is_left_as_names() {
        assert_eq!(scripted("a b").as_str(), "a;b");
        assert_eq!(scripted("").as_str(), "");
    }
}
