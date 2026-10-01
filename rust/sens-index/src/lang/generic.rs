use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

use tree_sitter::{Language, Node, Query, QueryCursor, StreamingIterator};

use super::treesitter::{Emitted, Extra, Options, Scope, declare, field, never_qualified, text};

const CALLABLE: [&str; 5] = ["function", "method", "fun_", "procedure", "subroutine"];
const HOLDING: [&str; 9] = ["class", "struct", "interface", "trait", "module", "object", "protocol", "enum", "impl"];
const DEFINING: [&str; 7] = ["definition", "declaration", "_def", "_decl", "statement", "item", "implementation"];
const BARE: [&str; 4] = ["function", "method", "class", "module"];
const ARGUMENTS: [&str; 2] = ["argument", "parameter"];

type Compiled = HashMap<(Language, &'static str), Option<Arc<Query>>>;

static QUERIES: LazyLock<Mutex<Compiled>> = LazyLock::new(Mutex::default);

#[derive(Clone, Copy, PartialEq)]
enum Role {
    Callable,
    Holding,
}

struct Found<'t> {
    node: Node<'t>,
    name: Node<'t>,
    role: Role,
}

fn role(kind: &str) -> Option<Role> {
    if CALLABLE.iter().any(|word| kind.contains(word)) {
        Some(Role::Callable)
    } else if HOLDING.iter().any(|word| kind.contains(word)) {
        Some(Role::Holding)
    } else {
        None
    }
}

fn shaped(node: &Node) -> Option<Role> {
    let kind = node.kind();
    let defining = DEFINING.iter().any(|word| kind.contains(word)) || (BARE.contains(&kind) && field(node, "name").is_some());
    defining.then(|| role(kind)).flatten()
}

fn named_like(kind: &str) -> bool {
    kind.contains("identifier") || kind.ends_with("name") || matches!(kind, "constant" | "word" | "atom" | "variable")
}

fn plain(said: &str) -> bool {
    !said.is_empty() && !said.starts_with('-') && said.chars().all(|c| c.is_alphanumeric() || matches!(c, '_' | '$' | '?' | '!' | '-'))
}

fn last_plain_leaf<'t>(node: Node<'t>, source: &str) -> Option<Node<'t>> {
    if node.named_child_count() == 0 {
        return (named_like(node.kind()) && plain(text(&node, source))).then_some(node);
    }
    let mut cursor = node.walk();
    let children: Vec<Node<'t>> = node.named_children(&mut cursor).collect();
    children.into_iter().rev().find_map(|child| last_plain_leaf(child, source))
}

fn first_plain_leaf<'t>(node: Node<'t>, source: &str) -> Option<Node<'t>> {
    let mut inner = node;
    loop {
        if ARGUMENTS.iter().any(|word| inner.kind().contains(word)) || shaped(&inner).is_some() {
            return None;
        }
        match inner.named_child(0) {
            Some(child) => inner = child,
            None => break,
        }
    }
    (named_like(inner.kind()) && plain(text(&inner, source))).then_some(inner)
}

fn name_of<'t>(node: &Node<'t>, source: &str) -> Option<Node<'t>> {
    let mut cursor = node.walk();
    match field(node, "name").or_else(|| node.named_children(&mut cursor).find(|child| named_like(child.kind()))) {
        Some(written) if plain(text(&written, source)) => Some(written),
        Some(written) => last_plain_leaf(written, source),
        None => first_plain_leaf(node.named_child(0)?, source),
    }
}

fn walk<'t>(node: Node<'t>, source: &str, out: &mut Vec<Found<'t>>) {
    if let Some(role) = shaped(&node)
        && let Some(name) = name_of(&node, source)
    {
        out.push(Found { node, name, role });
    }
    let mut cursor = node.walk();
    let children: Vec<Node<'t>> = node.named_children(&mut cursor).collect();
    for child in children {
        walk(child, source, out);
    }
}

fn compiled(root: &Node, definitions: &'static str) -> Option<Arc<Query>> {
    if definitions.is_empty() {
        return None;
    }
    let mut known = QUERIES.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    known.entry((Language::clone(&root.language()), definitions)).or_insert_with_key(|(language, definitions)| Query::new(language, definitions).ok().map(Arc::new)).clone()
}

fn child_holding<'t>(node: Node<'t>, name: Node<'t>) -> Node<'t> {
    let mut inner = name;
    while let Some(parent) = inner.parent() {
        if parent.id() == node.id() {
            return inner;
        }
        inner = parent;
    }
    node
}

fn tagged<'t>(root: &Node<'t>, source: &str, query: &Query) -> Vec<Found<'t>> {
    let labels = query.capture_names();
    let mut raw = Vec::new();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, *root, source.as_bytes());
    while let Some(matched) = matches.next() {
        let mut defined = None;
        let mut name = None;
        for capture in matched.captures() {
            let label = labels[capture.index as usize];
            if label == "name" {
                name = Some(capture.node);
            } else if let Some(role) = label.strip_prefix("definition.").and_then(role) {
                defined = Some((capture.node, role));
            }
        }
        if let (Some((node, role)), Some(name)) = (defined, name)
            && plain(text(&name, source))
        {
            raw.push(Found { node, name, role });
        }
    }
    let mut names_by_node: HashMap<usize, HashSet<usize>> = HashMap::new();
    for found in &raw {
        names_by_node.entry(found.node.id()).or_default().insert(found.name.id());
    }
    let mut narrowed: Vec<Found<'t>> = raw
        .into_iter()
        .map(|found| {
            let shared = found.node.parent().is_none() || names_by_node[&found.node.id()].len() > 1;
            let owns_name = found.name.parent().is_some_and(|parent| parent.id() == found.node.id());
            if shared && !owns_name { Found { node: child_holding(found.node, found.name), ..found } } else { found }
        })
        .collect();
    narrowed.sort_by_key(|found| found.node.end_byte() - found.node.start_byte());
    let (mut names, mut nodes) = (HashSet::new(), HashSet::new());
    narrowed.retain(|found| names.insert(found.name.id()) && nodes.insert(found.node.id()));
    narrowed
}

fn holder_of<'a>(found: &'a [Found], inner: &Found) -> Option<&'a Found<'a>> {
    found
        .iter()
        .filter(|outer| outer.role == Role::Holding && outer.node.id() != inner.node.id())
        .filter(|outer| outer.node.start_byte() <= inner.node.start_byte() && inner.node.end_byte() <= outer.node.end_byte())
        .max_by_key(|outer| outer.node.start_byte())
}

pub fn extract(root: &Node, source: &str, definitions: &'static str, out: &mut Emitted) {
    let mut found = match compiled(root, definitions) {
        Some(query) => tagged(root, source, &query),
        None => {
            let mut walked = Vec::new();
            walk(*root, source, &mut walked);
            walked
        }
    };
    found.sort_by_key(|definition| definition.node.start_byte());
    for definition in &found {
        let simple = text(&definition.name, source).to_string();
        let holder = holder_of(&found, definition).map(|outer| text(&outer.name, source));
        match (definition.role, holder) {
            (Role::Callable, Some(holder)) => {
                declare(out, source, format!("{holder}.{simple}"), "method", &definition.node, &definition.name, Extra { simple_name: Some(simple), ..Extra::default() })
            }
            (Role::Callable, None) => declare(out, source, simple, "function", &definition.node, &definition.name, Extra { exported: true, ..Extra::default() }),
            (Role::Holding, _) => declare(out, source, simple, "class", &definition.node, &definition.name, Extra { exported: true, ..Extra::default() }),
        }
    }
}

pub fn ocaml(file: &str) -> Language {
    if file.ends_with(".mli") { tree_sitter_ocaml::LANGUAGE_OCAML_INTERFACE.into() } else { tree_sitter_ocaml::LANGUAGE_OCAML.into() }
}

pub fn fsharp(file: &str) -> Language {
    if file.ends_with(".fsi") { tree_sitter_fsharp::LANGUAGE_SIGNATURE.into() } else { tree_sitter_fsharp::LANGUAGE_FSHARP.into() }
}

pub fn options() -> Options {
    Options { scope: Scope::Name, qualified_use: never_qualified }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbols(grammar: Language, definitions: &'static str, source: &str) -> Vec<(String, &'static str)> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&grammar).unwrap();
        let tree = parser.parse(source, None).unwrap();
        let mut out = Emitted::default();
        extract(&tree.root_node(), source, definitions, &mut out);
        out.symbols.into_iter().map(|symbol| (symbol.name, symbol.kind)).collect()
    }

    fn named(pairs: &[(&str, &'static str)]) -> Vec<(String, &'static str)> {
        pairs.iter().map(|&(name, kind)| (name.to_string(), kind)).collect()
    }

    const RUBY: &str = "class Shelf\n  def size\n    1\n  end\nend\n\ndef helper\nend\n";

    #[test]
    fn the_grammar_s_own_tags_name_classes_methods_and_functions() {
        let found = symbols(tree_sitter_ruby::LANGUAGE.into(), tree_sitter_ruby::TAGS_QUERY, RUBY);
        assert_eq!(found, named(&[("Shelf", "class"), ("Shelf.size", "method"), ("helper", "function")]));
    }

    #[test]
    fn a_function_inside_a_class_is_a_method_of_it() {
        let source = "class Box:\n    def open(self):\n        return 1\n\ndef close():\n    pass\n";
        let found = symbols(tree_sitter_python::LANGUAGE.into(), tree_sitter_python::TAGS_QUERY, source);
        assert_eq!(found, named(&[("Box", "class"), ("Box.open", "method"), ("close", "function")]));
    }

    #[test]
    fn without_tags_the_shape_of_the_nodes_finds_the_definitions() {
        let source = "package main\n\nfunc Run() int {\n\treturn 1\n}\n\nfunc (s Shelf) Size() int {\n\treturn 2\n}\n";
        let found = symbols(tree_sitter_go::LANGUAGE.into(), "", source);
        assert_eq!(found, named(&[("Run", "function"), ("Size", "function")]));
    }

    #[test]
    fn tags_that_do_not_compile_fall_back_to_the_shape() {
        let found = symbols(tree_sitter_ruby::LANGUAGE.into(), "((not a query", RUBY);
        assert_eq!(found, named(&[("Shelf", "class"), ("Shelf.size", "method"), ("helper", "function")]));
    }

    #[test]
    fn a_qualified_name_keeps_the_part_that_is_defined() {
        let source = "void Shelf::size() {}\n";
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_cpp::LANGUAGE.into()).unwrap();
        let tree = parser.parse(source, None).unwrap();
        let qualified = super::super::treesitter::first_descendant(&tree.root_node(), "qualified_identifier").unwrap();
        assert_eq!(last_plain_leaf(qualified, source).map(|leaf| text(&leaf, source)), Some("size"));
        assert!(plain("empty?") && !plain("Shelf::size"));
    }
}
