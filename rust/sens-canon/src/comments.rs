use std::collections::HashMap;

use sens_index::build;
use sens_index::lang::treesitter::text;
use tree_sitter::Node;

use crate::verdict::{Change, Finding, Rule, Severity};

fn collect<'s>(node: Node, source: &'s str, out: &mut Vec<(u32, &'s str)>) {
    if node.kind().contains("comment") {
        let said = text(&node, source);
        if !(node.start_byte() == 0 && said.starts_with("#!")) {
            out.push((node.start_position().row as u32 + 1, said.trim()));
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor).collect::<Vec<_>>() {
        collect(child, source, out);
    }
}

fn comments<'s>(path: &str, source: &'s str) -> Vec<(u32, &'s str)> {
    let mut found = Vec::new();
    if let Some((tree, _)) = build::parse(path, source) {
        collect(tree.root_node(), source, &mut found);
    }
    found
}

pub fn findings(change: &Change) -> Vec<Finding> {
    let Some(after) = change.after.as_deref() else {
        return Vec::new();
    };
    let mut known: HashMap<&str, usize> = HashMap::new();
    for (_, said) in comments(&change.path, change.before()) {
        *known.entry(said).or_default() += 1;
    }
    comments(&change.path, after)
        .into_iter()
        .filter(|(_, said)| match known.get_mut(said) {
            Some(left) if *left > 0 => {
                *left -= 1;
                false
            }
            _ => true,
        })
        .map(|(line, said)| Finding {
            rule: Rule::R6,
            severity: Severity::Block,
            file: change.path.clone(),
            line,
            message: format!("Line {line} of {} adds a comment. This project allows none: say it with names and structure instead, by renaming or by extracting a function whose name says it.", change.path),
            target: None,
            key: format!("R6:{}:{said}", change.path),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(path: &str, before: Option<&str>, after: &str) -> Change {
        Change { path: path.into(), before: before.map(str::to_string), after: Some(after.into()) }
    }

    #[test]
    fn a_new_comment_is_blocked_in_every_style() {
        let rust = change("src/lib.rs", None, "/// Adds.\npub fn add(a: u8) -> u8 {\n    // one more\n    a + 1 /* sure */\n}\n");
        let lines: Vec<u32> = findings(&rust).iter().map(|finding| finding.line).collect();
        assert_eq!(lines, [1, 3, 4]);
        let python = change("a.py", None, "def a():\n    # why\n    return 1\n");
        assert_eq!(findings(&python)[0].line, 2);
        assert_eq!(findings(&python)[0].severity, Severity::Block);
    }

    #[test]
    fn comments_already_there_and_the_shebang_are_not_the_turn_s() {
        let before = "#!/usr/bin/env node\n// old\nexport const a = 1;\n";
        let after = "#!/usr/bin/env node\n// old\nexport const a = 2;\n";
        assert!(findings(&change("bin/cli.js", Some(before), after)).is_empty());
        let one_more = "#!/usr/bin/env node\n// old\n// old\nexport const a = 2;\n";
        assert_eq!(findings(&change("bin/cli.js", Some(before), one_more)).len(), 1);
    }

    #[test]
    fn a_file_the_index_cannot_read_is_left_alone() {
        assert!(findings(&change("notes.md", None, "<!-- hi -->")).is_empty());
    }
}
