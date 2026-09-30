use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use tree_sitter::Node;

use crate::lang::treesitter::{EmitSymbol, text};

pub const MIN_TOKENS: usize = 30;
pub const SHINGLE: usize = 5;
pub const PERMUTATIONS: usize = 128;
pub const BANDS: usize = 32;
pub const WINDOW: usize = 4;
pub const NEAR: f32 = 0.8;
pub const TELLING_TEXT: usize = 12;
pub const SMALL_TOKENS: usize = 12;
pub const SMALL_WORDS: usize = 4;
pub const SMALL_NEAR: f32 = 0.7;
const GRAM: usize = 3;
const PLAIN_TYPES: [&str; 2] = ["primitive_type", "predefined_type"];
const BARE: [&str; 9] = ["this", "self", "super", "null", "undefined", "nil", "none", "true", "false"];

const ROWS: usize = PERMUTATIONS / BANDS;
pub const UNIT_KINDS: [&str; 3] = ["function", "method", "class"];
const KEPT_KINDS: [&str; 9] = ["property_identifier", "field_identifier", "type_identifier", "namespace_identifier", "package_identifier", "primitive_type", "predefined_type", "constant", "scoped_type_identifier"];
const CALLS: [&str; 4] = ["call", "invocation", "new_expression", "creation"];
const CALLEES: [&str; 5] = ["function", "method", "name", "constructor", "type"];
const MEMBERS: [&str; 7] = ["member", "attribute", "field", "selector", "navigation", "scoped", "access"];
const ACCESSED: [&str; 6] = ["property", "attribute", "field", "name", "member", "suffix"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Print {
    pub tokens: usize,
    pub exact: u64,
    pub renamed: u64,
    pub shingles: Vec<u64>,
    pub signature: Vec<u32>,
    pub grams: Vec<u64>,
    pub vocabulary: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub symbol: String,
    pub name: String,
    pub file: String,
    pub start_line: u32,
    pub end_line: u32,
    pub whole: bool,
    pub test: bool,
    pub callable: bool,
    pub print: Print,
}

impl Unit {
    pub fn comparable(&self) -> bool {
        self.print.tokens >= MIN_TOKENS
    }

    pub fn small(&self) -> bool {
        self.whole && self.callable && (SMALL_TOKENS..MIN_TOKENS).contains(&self.print.tokens) && self.print.vocabulary.len() >= SMALL_WORDS
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Likeness {
    Exact,
    Renamed,
    Near(f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Match {
    pub unit: usize,
    pub likeness: Likeness,
}

impl Match {
    pub fn similarity(&self) -> f32 {
        match self.likeness {
            Likeness::Exact | Likeness::Renamed => 1.0,
            Likeness::Near(share) => share,
        }
    }
}

struct Reader<'s> {
    source: &'s str,
    own_name: Option<(usize, &'s str)>,
    locals: HashMap<&'s str, usize>,
    raw: Vec<&'s str>,
    normalized: Vec<String>,
    vocabulary: Vec<&'s str>,
}

fn is_comment(kind: &str) -> bool {
    kind.contains("comment")
}

fn is_string(kind: &str) -> bool {
    (kind.contains("string") && !kind.contains("identifier") && !kind.ends_with("_type")) || kind.contains("char_literal") || matches!(kind, "character_literal" | "rune_literal" | "heredoc_body")
}

fn is_number(kind: &str) -> bool {
    ["number", "integer", "float", "decimal", "int_literal"].iter().any(|part| kind.contains(part))
}

fn is_name(kind: &str) -> bool {
    kind == "identifier" || kind.ends_with("_identifier") || matches!(kind, "name" | "constant" | "simple_identifier")
}

fn in_field(parent: &Node, node: &Node, fields: &[&str]) -> bool {
    fields.iter().any(|field| parent.child_by_field_name(field).is_some_and(|child| child.id() == node.id()))
}

fn kept(node: &Node) -> bool {
    if KEPT_KINDS.contains(&node.kind()) {
        return true;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    let kind = parent.kind();
    (CALLS.iter().any(|call| kind.contains(call)) && in_field(&parent, node, &CALLEES))
        || (MEMBERS.iter().any(|member| kind.contains(member)) && in_field(&parent, node, &ACCESSED))
}

impl<'s> Reader<'s> {
    fn new(source: &'s str, own_name: Option<(usize, &'s str)>) -> Self {
        Reader { source, own_name, locals: HashMap::new(), raw: Vec::new(), normalized: Vec::new(), vocabulary: Vec::new() }
    }

    fn read(&mut self, node: Node) {
        let kind = node.kind();
        if is_comment(kind) {
            return;
        }
        if node.is_named() && is_string(kind) {
            let literal = text(&node, self.source);
            self.raw.push(literal);
            self.vocabulary.push(literal);
            self.normalized.push(if literal.chars().count() >= TELLING_TEXT { literal.to_string() } else { "STR".into() });
            return;
        }
        if node.child_count() > 0 {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor).collect::<Vec<_>>() {
                self.read(child);
            }
            return;
        }
        let word = text(&node, self.source);
        if word.trim().is_empty() {
            return;
        }
        if self.own_name.is_some_and(|(start, _)| start == node.start_byte()) {
            return;
        }
        self.raw.push(word);
        if telling(&node, word) {
            self.vocabulary.push(word);
        }
        let normalized = if is_number(kind) {
            "NUM".into()
        } else if matches!(kind, "true" | "false" | "boolean" | "boolean_literal") {
            "BOOL".into()
        } else if !node.is_named() || !is_name(kind) {
            word.to_string()
        } else if kept(&node) {
            match self.own_name {
                Some((_, own)) if own == word => "$self".into(),
                _ => word.to_string(),
            }
        } else {
            let next = self.locals.len() + 1;
            format!("${}", self.locals.entry(word).or_insert(next))
        };
        self.normalized.push(normalized);
    }

    fn print(self) -> Print {
        Print { vocabulary: words_of(&self.vocabulary), ..print_of_tokens(hash_of(&self.raw), &self.normalized) }
    }
}

fn telling(node: &Node, word: &str) -> bool {
    let kind = node.kind();
    if !node.is_named() || BARE.contains(&word) || PLAIN_TYPES.contains(&kind) {
        return false;
    }
    if is_number(kind) || !is_name(kind) || kept(node) {
        return true;
    }
    word.chars().next().is_some_and(char::is_uppercase)
}

fn words_of(found: &[&str]) -> Vec<u64> {
    let mut hashed: Vec<u64> = found.iter().map(hash_of).collect();
    hashed.sort_unstable();
    hashed.dedup();
    hashed
}

fn grams_of(normalized: &[String]) -> Vec<u64> {
    let mut grams: Vec<u64> = normalized.windows(GRAM.min(normalized.len()).max(1)).map(hash_of).collect();
    grams.sort_unstable();
    grams.dedup();
    grams
}

pub fn print_of_tokens(exact: u64, normalized: &[String]) -> Print {
    let mut shingles: Vec<u64> = match normalized.len() {
        0 => Vec::new(),
        count if count < SHINGLE => vec![hash_of(normalized)],
        _ => normalized.windows(SHINGLE).map(hash_of).collect(),
    };
    shingles.sort_unstable();
    shingles.dedup();
    let signature = minhash(&shingles);
    Print { tokens: normalized.len(), exact, renamed: hash_of(normalized), shingles, signature, grams: grams_of(normalized), vocabulary: Vec::new() }
}

fn reading<'s>(nodes: &[Node], source: &'s str, own_name: Option<(usize, &'s str)>) -> Reader<'s> {
    let mut reader = Reader::new(source, own_name);
    for node in nodes {
        reader.read(*node);
    }
    reader
}

pub fn normalized(node: Node, source: &str, own_name: Option<(usize, &str)>) -> Vec<String> {
    reading(&[node], source, own_name).normalized
}

fn hash_of<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub fn mix(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn minhash(shingles: &[u64]) -> Vec<u32> {
    let mut signature = vec![u32::MAX; PERMUTATIONS];
    for shingle in shingles {
        for (at, slot) in signature.iter_mut().enumerate() {
            let value = mix(shingle ^ mix(at as u64)) as u32;
            if value < *slot {
                *slot = value;
            }
        }
    }
    signature
}

fn shared(a: &[u64], b: &[u64]) -> usize {
    let (mut left, mut right, mut shared) = (0, 0, 0);
    while left < a.len() && right < b.len() {
        match a[left].cmp(&b[right]) {
            std::cmp::Ordering::Equal => {
                shared += 1;
                left += 1;
                right += 1;
            }
            std::cmp::Ordering::Less => left += 1,
            std::cmp::Ordering::Greater => right += 1,
        }
    }
    shared
}

pub fn jaccard(a: &[u64], b: &[u64]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let both = shared(a, b);
    both as f32 / (a.len() + b.len() - both) as f32
}

pub fn overlap(a: &[u64], b: &[u64]) -> f32 {
    match a.len().min(b.len()) {
        0 => 0.0,
        smaller => shared(a, b) as f32 / smaller as f32,
    }
}

pub fn resemblance(a: &[u64], b: &[u64]) -> f32 {
    jaccard(a, b).max(overlap(a, b))
}

pub fn print_of(node: Node, source: &str, own_name: Option<(usize, &str)>) -> Print {
    reading(&[node], source, own_name).print()
}

fn statements<'t>(unit: &Node<'t>) -> Vec<Node<'t>> {
    let Some(body) = unit.child_by_field_name("body").or_else(|| unit.child_by_field_name("value").and_then(|value| value.child_by_field_name("body"))) else {
        return Vec::new();
    };
    let mut cursor = body.walk();
    body.named_children(&mut cursor).filter(|child| !is_comment(child.kind())).collect()
}

fn tested(node: &Node, source: &str) -> bool {
    let mut current = Some(*node);
    while let Some(item) = current {
        let mut sibling = item.prev_named_sibling();
        while let Some(attribute) = sibling.filter(|attribute| attribute.kind() == "attribute_item") {
            if text(&attribute, source).split(|c: char| !(c.is_alphanumeric() || c == '_')).any(|word| word == "test") {
                return true;
            }
            sibling = attribute.prev_named_sibling();
        }
        current = item.parent();
    }
    false
}

pub fn units(root: &Node, source: &str, file: &str, symbols: &[EmitSymbol], test: bool) -> Vec<Unit> {
    let mut found = Vec::new();
    for symbol in symbols.iter().filter(|symbol| UNIT_KINDS.contains(&symbol.kind)) {
        let Some(node) = root.descendant_for_byte_range(symbol.start, symbol.end) else {
            continue;
        };
        let own = own_name(source, symbol.name_start);
        let id = format!("{}#{}#{}", file, symbol.name, symbol.line);
        let test = test || tested(&node, source);
        let line = |at: &Node| at.start_position().row as u32 + 1;
        found.push(Unit {
            symbol: id.clone(),
            name: symbol.name.clone(),
            file: file.to_string(),
            start_line: line(&node),
            end_line: node.end_position().row as u32 + 1,
            whole: true,
            test,
            callable: symbol.kind != "class",
            print: print_of(node, source, own),
        });
        if symbol.kind == "class" {
            continue;
        }
        let body = statements(&node);
        if body.len() <= WINDOW {
            continue;
        }
        for window in body.windows(WINDOW) {
            found.push(Unit {
                symbol: id.clone(),
                name: symbol.name.clone(),
                file: file.to_string(),
                start_line: line(&window[0]),
                end_line: window[WINDOW - 1].end_position().row as u32 + 1,
                whole: false,
                test,
                callable: false,
                print: reading(window, source, None).print(),
            });
        }
    }
    found
}

pub fn own_name(source: &str, start: usize) -> Option<(usize, &str)> {
    let rest = source.get(start..)?;
    let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')).unwrap_or(rest.len());
    (end > 0).then(|| (start, &rest[..end]))
}

#[derive(Debug, Default)]
pub struct Clones {
    exact: HashMap<u64, Vec<usize>>,
    renamed: HashMap<u64, Vec<usize>>,
    bands: Vec<HashMap<u64, Vec<usize>>>,
    small: Vec<usize>,
}

fn band_key(signature: &[u32], band: usize) -> u64 {
    hash_of(&signature[band * ROWS..(band + 1) * ROWS])
}

impl Clones {
    pub fn of(units: &[Unit]) -> Clones {
        let mut clones = Clones { bands: vec![HashMap::new(); BANDS], ..Clones::default() };
        clones.small = units.iter().enumerate().filter(|(_, unit)| unit.small() && !unit.test).map(|(at, _)| at).collect();
        for (at, unit) in units.iter().enumerate().filter(|(_, unit)| unit.comparable()) {
            clones.exact.entry(unit.print.exact).or_default().push(at);
            clones.renamed.entry(unit.print.renamed).or_default().push(at);
            for band in 0..BANDS {
                clones.bands[band].entry(band_key(&unit.print.signature, band)).or_default().push(at);
            }
        }
        clones
    }

    pub fn similar(&self, units: &[Unit], print: &Print, threshold: f32) -> Vec<Match> {
        if print.tokens < MIN_TOKENS || self.bands.is_empty() {
            return Vec::new();
        }
        let mut seen: HashSet<usize> = HashSet::new();
        let mut found = Vec::new();
        for &at in self.exact.get(&print.exact).into_iter().flatten() {
            if seen.insert(at) {
                found.push(Match { unit: at, likeness: Likeness::Exact });
            }
        }
        for &at in self.renamed.get(&print.renamed).into_iter().flatten() {
            if seen.insert(at) {
                found.push(Match { unit: at, likeness: Likeness::Renamed });
            }
        }
        let candidates: HashSet<usize> = (0..BANDS).flat_map(|band| self.bands[band].get(&band_key(&print.signature, band)).into_iter().flatten().copied()).collect();
        let mut near: Vec<Match> = candidates
            .into_iter()
            .filter(|at| !seen.contains(at))
            .filter_map(|at| {
                let share = resemblance(&print.shingles, &units[at].print.shingles);
                (share >= threshold).then_some(Match { unit: at, likeness: Likeness::Near(share) })
            })
            .collect();
        near.sort_by(|a, b| b.similarity().total_cmp(&a.similarity()).then(a.unit.cmp(&b.unit)));
        found.extend(near);
        found
    }

    pub fn small_like(&self, units: &[Unit], print: &Print) -> Vec<Match> {
        if !(SMALL_TOKENS..MIN_TOKENS).contains(&print.tokens) || print.vocabulary.len() < SMALL_WORDS {
            return Vec::new();
        }
        let mut found: Vec<Match> = self.small.iter().filter_map(|&at| small_likeness(print, &units[at].print).map(|likeness| Match { unit: at, likeness })).collect();
        found.sort_by(|a, b| b.similarity().total_cmp(&a.similarity()).then(a.unit.cmp(&b.unit)));
        found
    }
}

pub fn small_likeness(print: &Print, other: &Print) -> Option<Likeness> {
    if print.exact == other.exact {
        return Some(Likeness::Exact);
    }
    let words = jaccard(&print.vocabulary, &other.vocabulary);
    if words < SMALL_NEAR {
        return None;
    }
    if print.renamed == other.renamed {
        return Some(Likeness::Renamed);
    }
    let share = resemblance(&print.grams, &other.grams).min(words);
    (share >= SMALL_NEAR).then_some(Likeness::Near(share))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parsed(language: tree_sitter::Language, source: &str) -> tree_sitter::Tree {
        let mut parser = Parser::new();
        parser.set_language(&language).unwrap();
        parser.parse(source, None).unwrap()
    }

    fn first_function(language: tree_sitter::Language, source: &str, kinds: &[&str]) -> Print {
        let tree = parsed(language, source);
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if kinds.contains(&node.kind()) {
                let name = node.child_by_field_name("name").map(|name| (name.start_byte(), text(&name, source)));
                return print_of(node, source, name);
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        panic!("no function in {source}")
    }

    fn typescript(source: &str) -> Print {
        first_function(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(), source, &["function_declaration"])
    }

    #[test]
    fn renamed_copies_share_a_print_and_real_copies_share_the_exact_one() {
        let one = typescript("function add(x: number) {\n  // sum\n  return x + 1;\n}\n");
        let spaced = typescript("function add(x: number) {   return x +   1; }");
        let renamed = typescript("function plus(value: number) { return value + 1; }");
        assert_eq!(one.exact, spaced.exact);
        assert_ne!(one.exact, renamed.exact);
        assert_eq!(one.renamed, renamed.renamed);
    }

    #[test]
    fn calls_and_members_keep_their_names() {
        let mapped = typescript("function a(list: number[], f: any) { return list.map(f); }");
        let filtered = typescript("function b(items: number[], g: any) { return items.filter(g); }");
        assert_ne!(mapped.renamed, filtered.renamed);
        let date = typescript("function a() { return new Date(); }");
        let map = typescript("function a() { return new Map(); }");
        assert_ne!(date.renamed, map.renamed);
        let called = typescript("function a(x: string) { return formatBytes(x); }");
        let other = typescript("function a(x: string) { return slugify(x); }");
        assert_ne!(called.renamed, other.renamed);
    }

    #[test]
    fn short_literals_are_reduced_to_their_type_and_telling_text_is_kept() {
        let one = typescript("function a() { return label(\"KB\", 1024, true); }");
        let other = typescript("function a() { return label(\"MB\", 2048, false); }");
        assert_eq!(one.renamed, other.renamed);
        assert_ne!(one.exact, other.exact);
        let declined = typescript("function a() { return say(\"The user declined it.\"); }");
        let stopped = typescript("function a() { return say(\"The user stopped the task.\"); }");
        assert_ne!(declined.renamed, stopped.renamed);
    }

    #[test]
    fn rust_tests_inside_a_source_file_are_known_as_tests() {
        let source = "fn live() -> u8 { 1 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks() { assert_eq!(live(), 1); }

    fn helper() {}
}
";
        let tree = parsed(tree_sitter_rust::LANGUAGE.into(), source);
        let mut emitted = crate::lang::treesitter::Emitted::default();
        crate::lang::rust::extract(&tree.root_node(), source, "src/lib.rs", &HashSet::new(), &mut emitted);
        let found = units(&tree.root_node(), source, "src/lib.rs", &emitted.symbols, false);
        let tested: Vec<(&str, bool)> = found.iter().map(|unit| (unit.name.as_str(), unit.test)).collect();
        assert_eq!(tested, [("live", false), ("checks", true), ("helper", true)]);
    }

    #[test]
    fn a_recursive_call_reads_as_itself_whatever_the_name() {
        let one = typescript("function fib(n: number): number { return n < 2 ? n : fib(n - 1) + fib(n - 2); }");
        let other = typescript("function fibo(k: number): number { return k < 2 ? k : fibo(k - 1) + fibo(k - 2); }");
        assert_eq!(one.renamed, other.renamed);
    }

    #[test]
    fn python_rust_and_go_copies_are_recognized_too() {
        let python = |source: &str| first_function(tree_sitter_python::LANGUAGE.into(), source, &["function_definition"]);
        assert_eq!(python("def a(x):\n    return x.strip().lower()\n").renamed, python("def b(y):\n    # tidy\n    return y.strip().lower()\n").renamed);
        assert_ne!(python("def a(x):\n    return x.strip()\n").renamed, python("def a(x):\n    return x.upper()\n").renamed);
        let rust = |source: &str| first_function(tree_sitter_rust::LANGUAGE.into(), source, &["function_item"]);
        assert_eq!(rust("fn a(v: &[u8]) -> usize { v.len() * 2 }").renamed, rust("fn b(bytes: &[u8]) -> usize { bytes.len() * 2 }").renamed);
        let go = |source: &str| first_function(tree_sitter_go::LANGUAGE.into(), source, &["function_declaration"]);
        assert_eq!(go("package p\nfunc a(s string) int { return len(s) + 1 }").renamed, go("package p\nfunc b(t string) int { return len(t) + 1 }").renamed);
    }

    #[test]
    fn a_near_copy_is_found_above_the_threshold_and_a_stranger_is_not() {
        let base = "function totals(rows: Row[]) {\n  let sum = 0;\n  let count = 0;\n  for (const row of rows) {\n    if (row.active) {\n      sum += row.amount;\n      count += 1;\n    }\n  }\n  return { sum, count, mean: count ? sum / count : 0 };\n}\n";
        let near = base.replace("  return", "  log(sum);\n  return");
        let stranger = "function render(user: User) {\n  const card = document.createElement('div');\n  card.className = 'card';\n  card.textContent = user.name;\n  document.body.appendChild(card);\n  return card;\n}\n";
        let unit = |source: &str| Unit { symbol: String::new(), name: String::new(), file: String::new(), start_line: 1, end_line: 1, whole: true, test: false, callable: true, print: typescript(source) };
        let units = vec![unit(base), unit(stranger)];
        assert!(units.iter().all(Unit::comparable));
        let clones = Clones::of(&units);
        let found = clones.similar(&units, &typescript(&near), NEAR);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].unit, 0);
        assert!(matches!(found[0].likeness, Likeness::Near(share) if share >= NEAR));
        let exact = clones.similar(&units, &typescript(base), NEAR);
        assert_eq!(exact[0], Match { unit: 0, likeness: Likeness::Exact });
    }

    #[test]
    fn small_pieces_are_never_compared() {
        let tiny = typescript("function a(x: number) { return x; }");
        assert!(tiny.tokens < MIN_TOKENS);
        let units = vec![Unit { symbol: String::new(), name: String::new(), file: String::new(), start_line: 1, end_line: 1, whole: true, test: false, callable: true, print: tiny.clone() }];
        assert!(Clones::of(&units).similar(&units, &tiny, NEAR).is_empty());
    }

    fn arrow(source: &str) -> Print {
        first_function(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(), source, &["arrow_function"])
    }

    fn small(print: Print) -> Unit {
        Unit { symbol: String::new(), name: String::new(), file: String::new(), start_line: 1, end_line: 1, whole: true, test: false, callable: true, print }
    }

    #[test]
    fn a_small_helper_written_again_is_found_by_what_it_calls() {
        let plain = arrow(r#"export const plain = (text) => text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();"#);
        let again = arrow(r#"const lower = (text: string) => text.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase();"#);
        let units = vec![small(plain)];
        assert!(units[0].small(), "{:?}", units[0].print.tokens);
        let found = Clones::of(&units).small_like(&units, &again);
        assert!(matches!(found.first().map(|found| found.likeness), Some(Likeness::Near(share)) if share >= SMALL_NEAR), "{found:?}");
    }

    #[test]
    fn small_helpers_that_only_share_a_shape_are_not_copies() {
        let known = vec![small(arrow("export const isFolder = (path: string) => FOLDER.test(path) && path.length > 0;")), small(arrow("export const setName = (name: string) => set({ name });"))];
        let clones = Clones::of(&known);
        assert!(clones.small_like(&known, &arrow("export const controlled = (text: string) => CONTROLS.test(text) && text.length > 0;")).is_empty());
        assert!(clones.small_like(&known, &arrow("export const setFilter = (filter: string) => set({ filter });")).is_empty());
    }

    #[test]
    fn jaccard_is_shared_over_union_and_overlap_is_shared_over_the_smaller() {
        assert_eq!(jaccard(&[1, 2, 3], &[2, 3, 4]), 0.5);
        assert_eq!(jaccard(&[], &[]), 1.0);
        assert_eq!(jaccard(&[1], &[2]), 0.0);
        assert_eq!(overlap(&[1, 2], &[1, 2, 3, 4, 5, 6]), 1.0);
        assert_eq!(overlap(&[], &[1]), 0.0);
        assert_eq!(resemblance(&[1, 2], &[1, 2, 3, 4]), 1.0);
    }
}
