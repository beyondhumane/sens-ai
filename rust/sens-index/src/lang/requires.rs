use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use super::treesitter::{Emitted, closest};

static DART: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?m)^\s*(?:import|export|part)\s+['"]([^'"]+)['"]"#).expect("a valid pattern"));
static LUA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\brequire\s*\(?\s*['"]([^'"]+)['"]"#).expect("a valid pattern"));

fn beside(file: &str, relative: &str) -> String {
    let mut parts: Vec<&str> = file.split('/').collect();
    parts.pop();
    for part in relative.split('/') {
        match part {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    parts.join("/")
}

pub fn dart(source: &str, file: &str, rel_set: &HashSet<String>, out: &mut Emitted) {
    for found in DART.captures_iter(source) {
        let uri = &found[1];
        let target = match uri.strip_prefix("package:") {
            Some(package) => package.split_once('/').map(|(_, inside)| format!("lib/{inside}")).filter(|inside| rel_set.contains(inside)),
            None if uri.starts_with("dart:") => None,
            None => closest(rel_set, &beside(file, uri)),
        };
        if let Some(target) = target {
            out.import(file, target, Vec::new());
        }
    }
}

pub fn lua(source: &str, file: &str, rel_set: &HashSet<String>, out: &mut Emitted) {
    for found in LUA.captures_iter(source) {
        let path = found[1].replace('.', "/");
        if let Some(target) = closest(rel_set, &format!("{path}.lua")).or_else(|| closest(rel_set, &format!("{path}/init.lua"))) {
            out.import(file, target, Vec::new());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Read = fn(&str, &str, &HashSet<String>, &mut Emitted);

    fn targets(read: Read, file: &str, source: &str, files: &[&str]) -> Vec<String> {
        let rel_set: HashSet<String> = files.iter().map(|file| file.to_string()).collect();
        let mut out = Emitted::default();
        read(source, file, &rel_set, &mut out);
        out.imports.into_iter().map(|edge| edge.to).collect()
    }

    #[test]
    fn dart_imports_reach_the_project_s_own_files_and_nothing_else() {
        let source = "import 'dart:async';\nimport 'package:flutter/material.dart';\nimport 'package:shelf/src/size.dart';\nimport '../widgets/card.dart';\npart 'shelf.g.dart';\n";
        let files = ["lib/src/size.dart", "lib/widgets/card.dart", "lib/screens/shelf.g.dart", "lib/screens/shelf.dart"];
        assert_eq!(targets(dart, "lib/screens/shelf.dart", source, &files), ["lib/src/size.dart", "lib/widgets/card.dart", "lib/screens/shelf.g.dart"]);
    }

    #[test]
    fn lua_requires_follow_the_dots_to_a_file_or_its_init() {
        let source = "local size = require(\"shelf.size\")\nlocal ui = require \"ui\"\nlocal json = require(\"cjson\")\n";
        let files = ["shelf/size.lua", "ui/init.lua", "main.lua"];
        assert_eq!(targets(lua, "main.lua", source, &files), ["shelf/size.lua", "ui/init.lua"]);
    }
}
