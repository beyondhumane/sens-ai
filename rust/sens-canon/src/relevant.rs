use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use sens_index::index::Index;
use sens_index::testfile::is_test_file;

const GLOSSARY: &str = include_str!("glossary.toml");
const LIMIT: usize = 8;
const K1: f32 = 1.2;
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
const SUFFIX: usize = 3;
const CODE_WORDS: [&str; 40] = [
    "const", "let", "var", "function", "return", "if", "else", "for", "while", "new", "this", "self", "fn", "pub", "def", "class", "true", "false", "null", "undefined",
    "await", "async", "import", "export", "from", "type", "void", "mut", "impl", "use", "match", "none", "some", "ok", "err", "string", "of", "in", "is", "as",
];
const MIN_SCORE: f32 = 0.1;
const KEEP_SHARE: f32 = 0.3;
const PREFIX: usize = 4;
const POPULARITY: f32 = 0.2;
const KINDS: [&str; 10] = ["function", "method", "class", "interface", "type", "enum", "struct", "trait", "const", "component"];
const STOP: [&str; 24] = ["the", "and", "for", "with", "that", "this", "from", "each", "into", "when", "los", "las", "del", "que", "una", "para", "con", "por", "cada", "como", "sus", "les", "des", "und"];

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
    terms: HashMap<String, usize>,
    length: usize,
    uses: usize,
}

pub struct Catalog {
    docs: Vec<Doc>,
    frequency: HashMap<String, usize>,
    average: f32,
}

fn glossary() -> &'static Vec<(String, Vec<String>)> {
    static TERMS: OnceLock<Vec<(String, Vec<String>)>> = OnceLock::new();
    TERMS.get_or_init(|| {
        let parsed: toml::Table = GLOSSARY.parse().unwrap_or_default();
        parsed
            .into_iter()
            .map(|(english, forms)| (english, forms.as_array().into_iter().flatten().filter_map(|form| form.as_str()).map(str::to_lowercase).collect()))
            .collect()
    })
}

fn stem(word: &str) -> String {
    if let Some(root) = word.strip_suffix("ies").filter(|root| root.len() >= 3) {
        return format!("{root}y");
    }
    match word.strip_suffix('s') {
        Some(root) if root.len() >= 3 && !root.ends_with('s') => root.to_string(),
        _ => word.to_string(),
    }
}

pub fn words(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut current = String::new();
    let mut previous: Option<char> = None;
    for c in text.chars() {
        let boundary = !c.is_alphanumeric() || (c.is_uppercase() && previous.is_some_and(|p| p.is_lowercase() || p.is_ascii_digit()));
        if boundary && !current.is_empty() {
            found.push(stem(&current.to_lowercase()));
            current.clear();
        }
        if c.is_alphanumeric() {
            current.push(c);
        }
        previous = Some(c);
    }
    if !current.is_empty() {
        found.push(stem(&current.to_lowercase()));
    }
    found
}

fn cjk(form: &str) -> bool {
    form.chars().any(|c| c as u32 >= 0x2E80)
}

fn shared_prefix(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

fn related(word: &str, form: &str) -> bool {
    let (word_length, form_length) = (word.chars().count(), form.chars().count());
    if form_length < PREFIX || word_length < PREFIX {
        return word == form;
    }
    let shorter = word_length.min(form_length);
    shared_prefix(word, form) >= PREFIX.max((shorter * 3).div_ceil(4))
}

fn translated(word: &str) -> impl Iterator<Item = String> + '_ {
    glossary().iter().filter(move |(_, forms)| forms.iter().any(|form| !cjk(form) && !form.contains(' ') && related(word, form))).map(|(english, _)| stem(english))
}

fn concepts(prompt: &str) -> Vec<HashSet<String>> {
    let lower = prompt.to_lowercase();
    let mut found: Vec<HashSet<String>> = Vec::new();
    let mut heard: HashSet<&str> = HashSet::new();
    for word in lower.split(|c: char| !c.is_alphanumeric()).filter(|word| word.chars().count() >= 2 && !STOP.contains(word)) {
        if !heard.insert(word) {
            continue;
        }
        let meant: HashSet<String> = translated(word).collect();
        if word.chars().count() >= 3 || !meant.is_empty() {
            found.push(words(word).into_iter().chain(meant).collect());
        }
    }
    for (english, forms) in glossary() {
        if forms.iter().any(|form| (cjk(form) || form.contains(' ')) && lower.contains(form.as_str())) {
            found.push(HashSet::from([stem(english)]));
        }
    }
    found
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
        let docs: Vec<Doc> = index
            .symbols
            .iter()
            .enumerate()
            .filter(|(_, symbol)| KINDS.contains(&symbol.kind.as_str()) && !is_test_file(&symbol.file) && !tested.contains(symbol.id.as_str()))
            .map(|(at, _)| {
                let terms = described(index, &mut sources, at);
                let length = terms.values().sum();
                Doc { symbol: at, file: index.symbols[at].file.clone(), name: index.symbols[at].name.clone(), terms, length, uses: index.raw_references(at).len() }
            })
            .collect();
        let mut frequency: HashMap<String, usize> = HashMap::new();
        for doc in &docs {
            for term in doc.terms.keys() {
                *frequency.entry(term.clone()).or_default() += 1;
            }
        }
        let average = docs.iter().map(|doc| doc.length).sum::<usize>() as f32 / docs.len().max(1) as f32;
        Catalog { docs, frequency, average }
    }

    fn matches(&self, terms: &HashSet<String>) -> HashSet<String> {
        self.frequency
            .keys()
            .filter(|known| terms.iter().any(|term| *known == term || (term.chars().count() >= PREFIX && known.starts_with(term.as_str()) && known.len() - term.len() <= SUFFIX)))
            .cloned()
            .collect()
    }

    fn wanted(&self, prompt: &str) -> Vec<HashSet<String>> {
        let mut groups: Vec<HashSet<String>> = Vec::new();
        for concept in concepts(prompt) {
            let matched = self.matches(&concept);
            if !matched.is_empty() && !groups.contains(&matched) {
                groups.push(matched);
            }
        }
        groups
    }

    fn weighed(&self, doc: &Doc, term: &str, flatten: f32) -> f32 {
        let Some(&count) = doc.terms.get(term) else {
            return 0.0;
        };
        let seen = self.frequency[term] as f32;
        let rarity = (1.0 + (self.docs.len() as f32 - seen + 0.5) / (seen + 0.5)).ln();
        let count = count as f32;
        rarity * count * (K1 + 1.0) / (count + K1 * (1.0 - flatten + flatten * doc.length as f32 / self.average))
    }

    fn ranked(&self, groups: &[HashSet<String>], flatten: f32, keep: impl Fn(&Doc) -> bool) -> Vec<Suggestion> {
        if groups.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<Suggestion> = self
            .docs
            .iter()
            .filter(|doc| keep(doc))
            .map(|doc| {
                let matched: f32 = groups.iter().map(|group| group.iter().map(|term| self.weighed(doc, term, flatten)).fold(0.0, f32::max)).sum();
                Suggestion { symbol: doc.symbol, score: matched * (1.0 + POPULARITY * (1.0 + doc.uses as f32).ln()), uses: doc.uses }
            })
            .collect();
        scored.sort_by(|a, b| b.score.total_cmp(&a.score).then(b.uses.cmp(&a.uses)).then(a.symbol.cmp(&b.symbol)));
        let best = scored.first().map_or(0.0, |top| top.score);
        scored.into_iter().filter(|found| found.score >= MIN_SCORE && found.score >= best * KEEP_SHARE).take(LIMIT).collect()
    }

    pub fn relevant(&self, prompt: &str) -> Vec<Suggestion> {
        self.ranked(&self.wanted(prompt), B, |_| true)
    }

    pub fn like_code(&self, code: &str, file: &str) -> Vec<Suggestion> {
        let spoken = spoken_code(code);
        let named: HashSet<&str> = spoken.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$')).filter(|name| !name.is_empty()).collect();
        let mut groups: Vec<HashSet<String>> = Vec::new();
        for word in meaningful(&spoken).collect::<HashSet<String>>() {
            let matched = self.matches(&HashSet::from([word]));
            if !matched.is_empty() && !groups.contains(&matched) {
                groups.push(matched);
            }
        }
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

    #[test]
    fn identifiers_split_into_stemmed_words() {
        assert_eq!(words("formatBytes"), ["format", "byte"]);
        assert_eq!(words("delete_user_entries"), ["delete", "user", "entry"]);
        assert_eq!(words("src/lib/format.ts"), ["src", "lib", "format", "ts"]);
        assert!(related("adjuntos", "adjunto") && related("salude", "saludar") && !related("carta", "carpeta") && !related("día", "dial"));
    }
}
