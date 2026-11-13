use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use sens_index::index::Index;
use sens_index::testfile::is_test_file;

use crate::rank::{self, words, Corpus};

const LIMIT: usize = 8;
const B: f32 = 0.75;
const B_CODE: f32 = 0.3;
const NAME_WEIGHT: usize = 3;
const SIGNATURE_WEIGHT: usize = 2;
const COMMENT_WEIGHT: usize = 2;
const CALLER_WEIGHT: usize = 1;
const BODY_WEIGHT: usize = 1;
const PATH_WEIGHT: usize = 1;
const CALLERS_READ: usize = 24;
const CALLER_CAP: usize = 3;
const CODE_WORDS: [&str; 40] = [
    "const", "let", "var", "function", "return", "if", "else", "for", "while", "new", "this", "self", "fn", "pub", "def", "class", "true", "false", "null", "undefined",
    "await", "async", "import", "export", "from", "type", "void", "mut", "impl", "use", "match", "none", "some", "ok", "err", "string", "of", "in", "is", "as",
];
const POPULARITY: f32 = 0.2;
const KINDS: [&str; 10] = ["function", "method", "class", "interface", "type", "enum", "struct", "trait", "const", "component"];

#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    pub symbol: usize,
    pub score: f32,
    pub uses: usize,
}

struct Doc {
    symbol: usize,
    file: String,
    name: String,
    uses: usize,
}

pub struct Catalog {
    docs: Vec<Doc>,
    corpus: Corpus,
}

fn quoted() -> &'static regex::Regex {
    static QUOTED: OnceLock<regex::Regex> = OnceLock::new();
    QUOTED.get_or_init(|| regex::Regex::new(r#""(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*'|`(?:[^`\\]|\\.)*`"#).unwrap())
}

fn unquoted(code: &str) -> String {
    quoted().replace_all(&code.replace("\"\"\"", " ").replace("'''", " "), " ").into_owned()
}

fn spoken_code(code: &str) -> String {
    quoted().replace_all(code, |found: &regex::Captures| interpolated(&found[0])).into_owned()
}

fn interpolated(literal: &str) -> String {
    static INSIDE: OnceLock<regex::Regex> = OnceLock::new();
    let inside = INSIDE.get_or_init(|| regex::Regex::new(r"\$\{([^{}]*)\}").unwrap());
    let kept: Vec<&str> = if literal.starts_with('`') { inside.captures_iter(literal).filter_map(|found| found.get(1)).map(|found| found.as_str()).collect() } else { Vec::new() };
    format!(" {} ", kept.join(" "))
}

fn remark(line: &str) -> bool {
    let line = line.trim_start();
    ["//", "#", "*", "/*", "--", "\"\"\""].iter().any(|mark| line.starts_with(mark)) || line.ends_with("*/")
}

fn comment_above(lines: &[&str], line: u32) -> String {
    let mut above: Vec<&str> = lines[..(line as usize).saturating_sub(1).min(lines.len())].iter().rev().take_while(|text| remark(text)).copied().collect();
    above.reverse();
    above.join("\n")
}

fn meaningful(text: &str) -> impl Iterator<Item = String> {
    words(text).into_iter().filter(|word| word.chars().count() >= 2 && !word.chars().all(|c| c.is_ascii_digit()) && !CODE_WORDS.contains(&word.as_str()))
}

fn add(terms: &mut HashMap<String, usize>, found: impl IntoIterator<Item = String>, weight: usize) {
    let distinct: HashSet<String> = found.into_iter().collect();
    for word in distinct {
        *terms.entry(word).or_default() += weight;
    }
}

struct Sources {
    root: std::path::PathBuf,
    read: HashMap<String, String>,
}

impl Sources {
    fn text(&mut self, file: &str) -> &str {
        let root = &self.root;
        self.read.entry(file.to_string()).or_insert_with(|| std::fs::read_to_string(root.join(file)).unwrap_or_default())
    }
}

fn callers(index: &Index, sources: &mut Sources, at: usize, own: &HashSet<String>) -> HashMap<String, usize> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    for &(file, line, _) in index.raw_references(at).iter().take(CALLERS_READ) {
        let path = index.files[file as usize].path.clone();
        if is_test_file(&path) {
            continue;
        }
        let Some(text) = sources.text(&path).lines().nth((line as usize).saturating_sub(1)).map(unquoted) else { continue };
        let words: HashSet<String> = meaningful(&text).filter(|word| !own.contains(word)).collect();
        for word in words {
            *seen.entry(word).or_default() += 1;
        }
    }
    seen
}

fn described(index: &Index, sources: &mut Sources, at: usize) -> HashMap<String, usize> {
    let symbol = &index.symbols[at];
    let named: HashSet<String> = words(&symbol.name).into_iter().collect();
    let mut terms: HashMap<String, usize> = HashMap::new();
    add(&mut terms, named.iter().cloned(), NAME_WEIGHT);
    add(&mut terms, meaningful(&unquoted(&symbol.signature)).filter(|word| !named.contains(word)), SIGNATURE_WEIGHT);
    for (word, sites) in callers(index, sources, at, &named) {
        *terms.entry(word).or_default() += CALLER_WEIGHT * sites.min(CALLER_CAP);
    }
    let text = sources.text(&symbol.file);
    let lines: Vec<&str> = text.lines().collect();
    add(&mut terms, meaningful(&comment_above(&lines, symbol.line)), COMMENT_WEIGHT);
    let body = text.get(symbol.start_byte..symbol.end_byte.min(text.len())).unwrap_or_default();
    add(&mut terms, meaningful(&unquoted(body)).filter(|word| !named.contains(word)), BODY_WEIGHT);
    add(&mut terms, meaningful(&symbol.file), PATH_WEIGHT);
    terms
}

impl Catalog {
    pub fn of(index: &Index) -> Catalog {
        let tested: HashSet<&str> = index.units.iter().filter(|unit| unit.whole && unit.test).map(|unit| unit.symbol.as_str()).collect();
        let mut sources = Sources { root: index.root.clone(), read: HashMap::new() };
        let (docs, terms): (Vec<Doc>, Vec<HashMap<String, usize>>) = index
            .symbols
            .iter()
            .enumerate()
            .filter(|(_, symbol)| KINDS.contains(&symbol.kind.as_str()) && !is_test_file(&symbol.file) && !tested.contains(symbol.id.as_str()))
            .map(|(at, symbol)| {
                let doc = Doc { symbol: at, file: symbol.file.clone(), name: symbol.name.clone(), uses: index.raw_references(at).len() };
                (doc, described(index, &mut sources, at))
            })
            .unzip();
        Catalog { docs, corpus: Corpus::of(terms) }
    }

    fn ranked(&self, groups: &[HashSet<String>], flatten: f32, keep: impl Fn(&Doc) -> bool) -> Vec<Suggestion> {
        if groups.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<Suggestion> = self
            .docs
            .iter()
            .enumerate()
            .filter(|(_, doc)| keep(doc))
            .map(|(at, doc)| Suggestion { symbol: doc.symbol, score: self.corpus.score(at, groups, flatten) * (1.0 + POPULARITY * (1.0 + doc.uses as f32).ln()), uses: doc.uses })
            .collect();
        scored.sort_by(|a, b| b.score.total_cmp(&a.score).then(b.uses.cmp(&a.uses)).then(a.symbol.cmp(&b.symbol)));
        rank::kept(scored, |found| found.score, LIMIT)
    }

    pub fn relevant(&self, prompt: &str) -> Vec<Suggestion> {
        self.ranked(&self.corpus.wanted(prompt), B, |_| true)
    }

    pub fn like_code(&self, code: &str, file: &str) -> Vec<Suggestion> {
        let spoken = spoken_code(code);
        let named: HashSet<&str> = spoken.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')).filter(|name| !name.is_empty()).collect();
        let groups = self.corpus.groups(meaningful(&spoken).collect::<HashSet<String>>().into_iter().map(|word| HashSet::from([word])));
        self.ranked(&groups, B_CODE, |doc| doc.file != file && !named.contains(doc.name.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::build;

    fn project(name: &str, files: &[(&str, &str)]) -> Index {
        let root = std::env::temp_dir().join("sens-canon-relevant").join(name);
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        build::build(&root)
    }

    fn names(index: &Index, prompt: &str) -> Vec<String> {
        Catalog::of(index).relevant(prompt).iter().map(|found| index.symbols[found.symbol].name.clone()).collect()
    }

    fn sample(name: &str) -> Index {
        project(
            name,
            &[
                ("src/greet.ts", "export function greet(name: string) { return `Hello, ${name}`; }\n"),
                ("src/users.ts", "export function deleteUser(id: string) { return id; }\nexport function createUser(name: string) { return name; }\n"),
                ("src/attachments.ts", "export interface Attachment { name: string }\nexport function listing(items: Attachment[]) { return items.length; }\n"),
                ("blog/text.py", "def slugify(title):\n    return title\n"),
                ("test/greet.test.ts", "export function greetInTest() {}\n"),
                ("src/lib.rs", "pub fn greeting() -> u8 { 1 }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn greeting_is_one() {}\n}\n"),
            ],
        )
    }

    #[test]
    fn a_request_in_spanish_finds_code_named_in_english() {
        let index = sample("spanish");
        assert_eq!(names(&index, "crea una función que salude a la persona").first().map(String::as_str), Some("greet"));
        assert_eq!(names(&index, "borra el usuario cuando se da de baja").first().map(String::as_str), Some("deleteUser"));
        assert!(names(&index, "Muestra el tamaño de cada adjunto en la lista").contains(&"Attachment".to_string()));
    }

    #[test]
    fn a_request_in_japanese_or_english_finds_it_too() {
        let index = sample("other");
        assert_eq!(names(&index, "ユーザーを削除する").first().map(String::as_str), Some("deleteUser"));
        assert_eq!(names(&index, "Fix the slugs of titles with accents").first().map(String::as_str), Some("slugify"));
    }

    #[test]
    fn small_talk_finds_nothing_and_tests_are_never_offered() {
        let index = sample("quiet");
        assert!(names(&index, "hola, ¿qué tal?").is_empty());
        let greeted = names(&index, "greet everyone");
        assert!(!greeted.contains(&"greetInTest".to_string()) && !greeted.contains(&"greeting_is_one".to_string()), "{greeted:?}");
    }

    #[test]
    fn new_code_finds_what_it_reinvents_elsewhere_but_not_what_it_already_calls() {
        let index = project(
            "code",
            &[
                ("src/market/search.js", r#"export const plain = (text) => text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();"#),
                ("src/market/use.js", "import { plain } from './search.js';\nexport const matching = (items, query) => items.filter((item) => plain(item.name).includes(plain(query)));\n"),
                ("src/format.js", "export const weigh = (bytes) => `${Math.round(bytes / 1024)} KB`;\n"),
            ],
        );
        let catalog = Catalog::of(&index);
        let named = |code: &str| catalog.like_code(code, "src/bar/choices.ts").iter().map(|found| index.symbols[found.symbol].name.clone()).collect::<Vec<_>>();
        let rewritten = r#"const lower = (text: string) => text.normalize("NFD").replace(/\p{M}/gu, "").toLocaleLowerCase();"#;
        assert_eq!(named(rewritten).first().map(String::as_str), Some("plain"));
        assert!(!named("const wanted = plain(filter.trim()).normalize(\"NFD\");").contains(&"plain".to_string()));
    }
}
