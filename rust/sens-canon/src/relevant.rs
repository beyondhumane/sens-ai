use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use sens_index::index::Index;
use sens_index::testfile::is_test_file;

const GLOSSARY: &str = include_str!("glossary.toml");
const LIMIT: usize = 8;
const K1: f32 = 1.2;
const B: f32 = 0.75;
const NAME_WEIGHT: usize = 3;
const MIN_SCORE: f32 = 0.1;
const KEEP_SHARE: f32 = 0.3;
const PREFIX: usize = 4;
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
    terms: HashMap<String, usize>,
    length: usize,
    named: HashSet<String>,
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

fn query(prompt: &str) -> HashSet<String> {
    let lower = prompt.to_lowercase();
    let spoken: Vec<&str> = lower.split(|c: char| !c.is_alphanumeric()).filter(|word| !word.is_empty()).collect();
    let mut terms: HashSet<String> = words(prompt).into_iter().filter(|word| word.chars().count() >= 3 && !STOP.contains(&word.as_str())).collect();
    for (english, forms) in glossary() {
        let meant = forms.iter().any(|form| match (cjk(form), form.contains(' ')) {
            (true, _) | (_, true) => lower.contains(form.as_str()),
            _ => spoken.iter().any(|word| related(word, form)),
        });
        if meant {
            terms.insert(stem(english));
        }
    }
    terms
}

impl Catalog {
    pub fn of(index: &Index) -> Catalog {
        let tested: HashSet<&str> = index.units.iter().filter(|unit| unit.whole && unit.test).map(|unit| unit.symbol.as_str()).collect();
        let docs: Vec<Doc> = index
            .symbols
            .iter()
            .enumerate()
            .filter(|(_, symbol)| KINDS.contains(&symbol.kind.as_str()) && !is_test_file(&symbol.file) && !tested.contains(symbol.id.as_str()))
            .map(|(at, symbol)| {
                let named: HashSet<String> = words(&symbol.name).into_iter().collect();
                let mut terms: HashMap<String, usize> = HashMap::new();
                for word in &named {
                    *terms.entry(word.clone()).or_default() += NAME_WEIGHT;
                }
                for word in words(&symbol.file) {
                    *terms.entry(word).or_default() += 1;
                }
                let length = terms.values().sum();
                Doc { symbol: at, terms, length, named, uses: index.raw_references(at).len() }
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
            .filter(|known| terms.iter().any(|term| *known == term || (term.chars().count() >= PREFIX && known.starts_with(term.as_str()))))
            .cloned()
            .collect()
    }

    pub fn relevant(&self, prompt: &str) -> Vec<Suggestion> {
        let wanted = self.matches(&query(prompt));
        if wanted.is_empty() {
            return Vec::new();
        }
        let total = self.docs.len() as f32;
        let mut scored: Vec<Suggestion> = self
            .docs
            .iter()
            .filter(|doc| doc.named.iter().any(|word| wanted.contains(word)))
            .map(|doc| {
                let score = doc
                    .terms
                    .iter()
                    .filter(|(term, _)| wanted.contains(*term))
                    .map(|(term, &count)| {
                        let seen = self.frequency[term] as f32;
                        let rarity = (1.0 + (total - seen + 0.5) / (seen + 0.5)).ln();
                        let count = count as f32;
                        rarity * count * (K1 + 1.0) / (count + K1 * (1.0 - B + B * doc.length as f32 / self.average))
                    })
                    .sum();
                Suggestion { symbol: doc.symbol, score, uses: doc.uses }
            })
            .collect();
        scored.sort_by(|a, b| b.score.total_cmp(&a.score).then(b.uses.cmp(&a.uses)).then(a.symbol.cmp(&b.symbol)));
        let best = scored.first().map_or(0.0, |top| top.score);
        scored.into_iter().filter(|found| found.score >= MIN_SCORE && found.score >= best * KEEP_SHARE).take(LIMIT).collect()
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
    fn identifiers_split_into_stemmed_words() {
        assert_eq!(words("formatBytes"), ["format", "byte"]);
        assert_eq!(words("delete_user_entries"), ["delete", "user", "entry"]);
        assert_eq!(words("src/lib/format.ts"), ["src", "lib", "format", "ts"]);
        assert!(related("adjuntos", "adjunto") && related("salude", "saludar") && !related("carta", "carpeta") && !related("día", "dial"));
    }
}
