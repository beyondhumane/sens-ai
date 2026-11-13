use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

const GLOSSARY: &str = include_str!("glossary.toml");
const K1: f32 = 1.2;
const MIN_SCORE: f32 = 0.1;
const KEEP_SHARE: f32 = 0.3;
const PREFIX: usize = 4;
const SUFFIX: usize = 3;
const STOP: [&str; 24] = ["the", "and", "for", "with", "that", "this", "from", "each", "into", "when", "los", "las", "del", "que", "una", "para", "con", "por", "cada", "como", "sus", "les", "des", "und"];

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

pub struct Corpus {
    terms: Vec<HashMap<String, usize>>,
    lengths: Vec<usize>,
    frequency: HashMap<String, usize>,
    average: f32,
}

impl Corpus {
    pub fn of(terms: Vec<HashMap<String, usize>>) -> Corpus {
        let lengths: Vec<usize> = terms.iter().map(|doc| doc.values().sum()).collect();
        let mut frequency: HashMap<String, usize> = HashMap::new();
        for doc in &terms {
            for term in doc.keys() {
                *frequency.entry(term.clone()).or_default() += 1;
            }
        }
        let average = lengths.iter().sum::<usize>() as f32 / terms.len().max(1) as f32;
        Corpus { terms, lengths, frequency, average }
    }

    fn matches(&self, terms: &HashSet<String>) -> HashSet<String> {
        self.frequency
            .keys()
            .filter(|known| terms.iter().any(|term| *known == term || (term.chars().count() >= PREFIX && known.starts_with(term.as_str()) && known.len() - term.len() <= SUFFIX)))
            .cloned()
            .collect()
    }

    pub fn groups(&self, concepts: impl IntoIterator<Item = HashSet<String>>) -> Vec<HashSet<String>> {
        let mut groups: Vec<HashSet<String>> = Vec::new();
        for concept in concepts {
            let matched = self.matches(&concept);
            if !matched.is_empty() && !groups.contains(&matched) {
                groups.push(matched);
            }
        }
        groups
    }

    pub fn wanted(&self, prompt: &str) -> Vec<HashSet<String>> {
        self.groups(concepts(prompt))
    }

    fn weighed(&self, doc: usize, term: &str, flatten: f32) -> f32 {
        let Some(&count) = self.terms[doc].get(term) else {
            return 0.0;
        };
        let seen = self.frequency[term] as f32;
        let rarity = (1.0 + (self.terms.len() as f32 - seen + 0.5) / (seen + 0.5)).ln();
        let count = count as f32;
        rarity * count * (K1 + 1.0) / (count + K1 * (1.0 - flatten + flatten * self.lengths[doc] as f32 / self.average))
    }

    pub fn score(&self, doc: usize, groups: &[HashSet<String>], flatten: f32) -> f32 {
        groups.iter().map(|group| group.iter().map(|term| self.weighed(doc, term, flatten)).fold(0.0, f32::max)).sum()
    }
}

pub fn kept<T>(ranked: Vec<T>, score: impl Fn(&T) -> f32, limit: usize) -> Vec<T> {
    let best = ranked.first().map_or(0.0, &score);
    ranked.into_iter().filter(|found| score(found) >= MIN_SCORE && score(found) >= best * KEEP_SHARE).take(limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECTIONS: [(&str, &str); 4] = [
        ("Jump", "Variable jump height, coyote time and jump buffering make a jump feel fair."),
        ("Wave spawning", "Enemies arrive in waves, from spawn points, with pacing between one wave and the next."),
        ("Inventory", "Slots, stacking and sorting of the items the player carries."),
        ("Camera", "Look-ahead, dead zones and smoothing so the camera follows the player."),
    ];

    fn counted(text: &str) -> HashMap<String, usize> {
        words(text).into_iter().fold(HashMap::new(), |mut terms, word| {
            *terms.entry(word).or_default() += 1;
            terms
        })
    }

    fn best(prompt: &str) -> Option<&'static str> {
        let corpus = Corpus::of(SECTIONS.iter().map(|(title, body)| counted(&format!("{title}. {body}"))).collect());
        let groups = corpus.wanted(prompt);
        let mut scored: Vec<(usize, f32)> = (0..SECTIONS.len()).map(|at| (at, corpus.score(at, &groups, 0.75))).collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));
        kept(scored, |found| found.1, 3).first().map(|&(at, _)| SECTIONS[at].0)
    }

    #[test]
    fn a_request_in_any_language_finds_the_game_section_written_in_english() {
        assert_eq!(best("el salto"), Some("Jump"));
        assert_eq!(best("añade doble salto al jugador"), Some("Jump"));
        assert_eq!(best("las oleadas llegan demasiado juntas"), Some("Wave spawning"));
        assert_eq!(best("ジャンプを調整する"), Some("Jump"));
        assert_eq!(best("Kamera einstellen"), Some("Camera"));
        assert_eq!(best("hola, ¿qué tal?"), None);
    }

    #[test]
    fn the_glossary_parses_and_every_entry_has_forms() {
        let parsed: toml::Table = GLOSSARY.parse().expect("glossary.toml is not valid TOML, and a broken glossary would silently turn into an empty one");
        assert_eq!(parsed.len(), glossary().len());
        for (english, forms) in &parsed {
            let forms = forms.as_array().unwrap_or_else(|| panic!("{english} has no list of forms"));
            assert!(!forms.is_empty() && forms.iter().all(|form| form.as_str().is_some_and(|form| !form.trim().is_empty())), "{english}");
        }
    }

    #[test]
    fn identifiers_split_into_stemmed_words() {
        assert_eq!(words("formatBytes"), ["format", "byte"]);
        assert_eq!(words("delete_user_entries"), ["delete", "user", "entry"]);
        assert_eq!(words("src/lib/format.ts"), ["src", "lib", "format", "ts"]);
        assert!(related("adjuntos", "adjunto") && related("salude", "saludar") && !related("carta", "carpeta") && !related("día", "dial"));
    }
}
