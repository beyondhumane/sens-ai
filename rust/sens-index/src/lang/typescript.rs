use std::collections::HashSet;

use tree_sitter::Node;

use super::component;
use super::treesitter::{Emitted, Extra, Options, Scope, all_named, declare, descendants, field, named, never_qualified, text};
use crate::index::EVERYTHING;

const RESOLVED: [&str; 13] = ["", ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", "/index.ts", "/index.tsx", "/index.js", "/index.jsx"];
const WRITTEN_AS_JS: [&str; 4] = [".js", ".jsx", ".mjs", ".cjs"];

pub fn grammar(file: &str) -> tree_sitter::Language {
    match file.rsplit('.').next().unwrap_or_default() {
        "tsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        "ts" | "mts" | "cts" | "vue" | "svelte" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        _ => tree_sitter_javascript::LANGUAGE.into(),
    }
}

pub fn prepare(file: &str, source: String) -> String {
    match file.rsplit('.').next().unwrap_or_default() {
        "vue" | "svelte" => component::scripted(&source),
        _ => source,
    }
}

pub fn options() -> Options {
    Options { scope: Scope::Import, qualified_use: never_qualified }
}

pub fn extract(root: &Node, source: &str, file: &str, files: &HashSet<String>, out: &mut Emitted) {
    let mut cursor = root.walk();
    for node in root.named_children(&mut cursor).collect::<Vec<Node>>() {
        match node.kind() {
            "export_statement" => exported(&node, source, file, files, out),
            "import_statement" => import(&node, source, file, files, out, "import_specifier"),
            _ => declaration(&node, source, out, false),
        }
    }
}

fn exported(node: &Node, source: &str, file: &str, files: &HashSet<String>, out: &mut Emitted) {
    if field(node, "source").is_some() {
        import(node, source, file, files, out, "export_specifier");
    }
    let Some(declared) = field(node, "declaration") else {
        return;
    };
    declaration(&declared, source, out, true);
}

pub fn resolve(file: &str, specifier: &str, files: &HashSet<String>) -> Option<String> {
    if !specifier.starts_with('.') {
        return None;
    }
    let mut parts: Vec<&str> = file.split('/').collect();
    parts.pop();
    for segment in specifier.split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    let written = parts.join("/");
    let stems = std::iter::once(written.clone()).chain(WRITTEN_AS_JS.iter().find_map(|js| written.strip_suffix(js)).map(str::to_string));
    stems.flat_map(|stem| RESOLVED.iter().map(move |ending| format!("{stem}{ending}"))).find(|candidate| files.contains(candidate))
}

fn declaration(node: &Node, source: &str, out: &mut Emitted, exported: bool) {
    match node.kind() {
        "function_declaration" | "generator_function_declaration" => {
            named_symbol(node, source, out, "function", exported);
        }
        "class_declaration" | "abstract_class_declaration" => {
            named_symbol(node, source, out, "class", exported);
            methods(node, source, out);
        }
        "interface_declaration" => named_symbol(node, source, out, "interface", exported),
        "type_alias_declaration" => named_symbol(node, source, out, "type", exported),
        "enum_declaration" => named_symbol(node, source, out, "enum", exported),
        "lexical_declaration" | "variable_declaration" => {
            bindings(node, source, out, exported);
        }
        _ => {}
    }
}

fn named_symbol(node: &Node, source: &str, out: &mut Emitted, kind: &'static str, exported: bool) {
    let Some(name_node) = field(node, "name") else {
        return;
    };
    let name = text(&name_node, source).to_string();
    declare(out, source, name, kind, node, &name_node, Extra { exported, ..Extra::default() });
}

fn bindings(node: &Node, source: &str, out: &mut Emitted, exported: bool) {
    for declarator in all_named(node, "variable_declarator") {
        let Some(name_node) = field(&declarator, "name") else {
            continue;
        };
        if name_node.kind() != "identifier" {
            continue;
        }
        let name = text(&name_node, source).to_string();
        let kind = match field(&declarator, "value").map(|value| value.kind()) {
            Some("arrow_function" | "function_expression" | "generator_function") => "function",
            _ => "const",
        };
        declare(out, source, name, kind, &declarator, &name_node, Extra { exported, ..Extra::default() });
    }
}

fn methods(class: &Node, source: &str, out: &mut Emitted) {
    let Some(name_node) = field(class, "name") else {
        return;
    };
    let owner = text(&name_node, source).to_string();
    let Some(body) = named(class, "class_body") else {
        return;
    };

    for member in all_named(&body, "method_definition") {
        let Some(member_name) = field(&member, "name") else {
            continue;
        };
        let simple = text(&member_name, source).to_string();
        declare(out, source, format!("{owner}.{simple}"), "method", &member, &member_name, Extra { exported: false, simple_name: Some(simple),
                ..Extra::default()
            });
    }
}

fn import(node: &Node, source: &str, file: &str, files: &HashSet<String>, out: &mut Emitted, specifier: &str) {
    let Some(from) = field(node, "source") else {
        return;
    };
    let written = text(&from, source).trim_matches(|c| c == '"' || c == '\'' || c == '`');
    let Some(target) = resolve(file, written, files) else {
        return;
    };
    let mut names: Vec<String> = descendants(node, specifier)
        .iter()
        .filter_map(|found| field(found, "name"))
        .map(|name| text(&name, source).to_string())
        .collect();
    if node.kind() == "export_statement" && names.is_empty() {
        names.push(EVERYTHING.into());
    }
    out.import(file, target, names);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn symbols(source: &str) -> Vec<(String, &'static str, bool)> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .unwrap();
        let tree = parser.parse(source, None).unwrap();
        let mut out = Emitted::default();
        extract(&tree.root_node(), source, "a.ts", &HashSet::new(), &mut out);
        out.symbols
            .into_iter()
            .map(|symbol| (symbol.name, symbol.kind, symbol.exported))
            .collect()
    }

    #[test]
    fn an_exported_function_is_found_and_marked_exported() {
        let found = symbols("export function parseConfig(raw: string): Config {\n  return {};\n}\n");
        assert_eq!(found, vec![("parseConfig".to_string(), "function", true)]);
    }

    #[test]
    fn a_private_function_is_found_but_not_exported() {
        let found = symbols("function helper() {}\n");
        assert_eq!(found, vec![("helper".to_string(), "function", false)]);
    }

    #[test]
    fn an_arrow_constant_counts_as_a_function() {
        let found = symbols("export const shorten = (s: string) => s.trim();\n");
        assert_eq!(found, vec![("shorten".to_string(), "function", true)]);
    }

    #[test]
    fn a_plain_constant_is_not_mistaken_for_a_function() {
        let found = symbols("export const LIMIT = 40;\n");
        assert_eq!(found, vec![("LIMIT".to_string(), "const", true)]);
    }

    #[test]
    fn types_interfaces_and_enums_all_come_through() {
        let found = symbols(
            "export interface Config { root: string }\nexport type Name = string;\nexport enum Tier { High }\n",
        );
        let kinds: Vec<&str> = found.iter().map(|(_, kind, _)| *kind).collect();
        assert_eq!(kinds, vec!["interface", "type", "enum"]);
    }

    #[test]
    fn a_class_brings_its_methods_with_it() {
        let found = symbols("export class Engine {\n  findSymbol(name: string) {}\n  private warm() {}\n}\n");
        let names: Vec<String> = found.iter().map(|(name, _, _)| name.clone()).collect();

        assert_eq!(names, vec!["Engine", "Engine.findSymbol", "Engine.warm"]);
    }

    #[test]
    fn a_destructured_binding_is_skipped_instead_of_invented() {
        let found = symbols("const { a, b } = thing;\n");
        assert!(found.is_empty());
    }
}
