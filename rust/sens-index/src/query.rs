use std::collections::{HashMap, HashSet};
use std::path::MAIN_SEPARATOR;

use crate::index::{Index, Reference, SymbolInfo};
use crate::reflective;
use crate::testfile::is_test_file;

pub struct Engine<'a> {
    index: &'a Index,
    entry_points: HashSet<&'a str>,
    by_id: HashMap<&'a str, usize>,
    by_name_lower: HashMap<String, Vec<usize>>,
    by_name_or_suffix: HashMap<&'a str, Vec<usize>>,
    by_file: Vec<(&'a str, Vec<usize>)>,
    by_file_index: HashMap<&'a str, usize>,
    imported: HashSet<&'a str>,
    callees_of: HashMap<usize, OrderedSet>,
}

#[derive(Default)]
pub struct OrderedSet {
    order: Vec<usize>,
    seen: HashSet<usize>,
}

impl OrderedSet {
    fn insert(&mut self, value: usize) {
        if self.seen.insert(value) {
            self.order.push(value);
        }
    }
    fn iter(&self) -> impl Iterator<Item = &usize> {
        self.order.iter()
    }
}

pub struct WhoUses<'a> {
    pub symbol: &'a SymbolInfo,
    pub references: Vec<Reference>,
}

pub struct MapEntry<'a> {
    pub file: &'a str,
    pub exported: Vec<&'a SymbolInfo>,
    pub internal_count: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Tier {
    High,
    Medium,
    Low,
}

pub struct DeadCandidate<'a> {
    pub symbol: &'a SymbolInfo,
    pub tier: Tier,
    pub reason: &'static str,
    pub reflective_hit: Option<String>,
}

pub struct DeadCodeReport<'a> {
    pub candidates: Vec<DeadCandidate<'a>>,
    pub files: Vec<&'a str>,
}

pub fn compare_paths(a: &str, b: &str) -> std::cmp::Ordering {
    a.cmp(b)
}

fn normalize(path: &str) -> String {
    path.replace(MAIN_SEPARATOR, "/")
}

impl<'a> Engine<'a> {
    pub fn new(index: &'a Index, entry_points: &'a [String]) -> Self {
        let mut engine = Self {
            index,
            entry_points: entry_points.iter().map(String::as_str).collect(),
            by_id: HashMap::new(),
            by_name_lower: HashMap::new(),
            by_name_or_suffix: HashMap::new(),
            by_file: Vec::new(),
            by_file_index: HashMap::new(),
            imported: HashSet::new(),
            callees_of: HashMap::new(),
        };

        for (i, s) in index.symbols.iter().enumerate() {
            engine.by_id.insert(s.id.as_str(), i);
            engine.by_name_lower.entry(s.name.to_lowercase()).or_default().push(i);
            engine.by_name_or_suffix.entry(s.name.as_str()).or_default().push(i);
            if let Some(dot) = s.name.rfind('.') {
                engine.by_name_or_suffix.entry(&s.name[dot + 1..]).or_default().push(i);
            }
            match engine.by_file_index.get(s.file.as_str()) {
                Some(&slot) => engine.by_file[slot].1.push(i),
                None => {
                    engine.by_file_index.insert(s.file.as_str(), engine.by_file.len());
                    engine.by_file.push((s.file.as_str(), vec![i]));
                }
            }
        }

        engine.imported = index.imports.iter().filter(|edge| edge.from != edge.to).map(|edge| edge.to.as_str()).collect();

        for si in 0..index.symbols.len() {
            for &(_, _, from) in index.raw_references(si) {
                if from == u32::MAX || from as usize == si {
                    continue;
                }
                engine.callees_of.entry(from as usize).or_default().insert(si);
            }
        }

        engine
    }

    fn symbol(&self, i: usize) -> &'a SymbolInfo {
        &self.index.symbols[i]
    }

    fn resolve_indices(&self, name: &str) -> Vec<usize> {
        self.by_name_or_suffix.get(name).cloned().unwrap_or_default()
    }

    fn resolve(&self, name: &str) -> Vec<&'a SymbolInfo> {
        self.resolve_indices(name).into_iter().map(|i| self.symbol(i)).collect()
    }

    pub fn find_symbol(&self, name: &str) -> Vec<&'a SymbolInfo> {
        self.by_name_lower
            .get(&name.to_lowercase())
            .map(|ids| ids.iter().map(|&i| self.symbol(i)).collect())
            .unwrap_or_default()
    }

    pub fn who_uses(&self, name: &str) -> Vec<WhoUses<'a>> {
        self.resolve(name)
            .into_iter()
            .map(|symbol| WhoUses {
                references: self.index.references_for(&symbol.id),
                symbol,
            })
            .collect()
    }

    pub fn file_outline(&self, file: &str) -> Vec<&'a SymbolInfo> {
        let norm = normalize(file);
        let mut syms: Vec<&SymbolInfo> = match self.by_file_index.get(norm.as_str()) {
            Some(&slot) => self.by_file[slot].1.iter().map(|&i| self.symbol(i)).collect(),
            None => self.index.symbols.iter().filter(|s| s.file.ends_with(&norm)).collect(),
        };
        syms.sort_by_key(|s| s.line);
        syms
    }
}

impl<'a> Engine<'a> {
    pub fn map(&self, subdir: Option<&str>) -> Vec<MapEntry<'a>> {
        let sub = subdir.map(normalize);
        let mut entries: Vec<MapEntry> = Vec::new();
        for (file, ids) in &self.by_file {
            if let Some(sub) = &sub
                && !file.starts_with(sub.as_str()) {
                    continue;
                }
            let mut exported: Vec<&SymbolInfo> = ids
                .iter()
                .map(|&i| self.symbol(i))
                .filter(|s| s.exported && s.kind != "method")
                .collect();
            exported.sort_by_key(|s| s.line);
            entries.push(MapEntry {
                file,
                internal_count: ids.len() - exported.len(),
                exported,
            });
        }
        entries.sort_by(|a, b| compare_paths(a.file, b.file));
        entries
    }
}

impl<'a> Engine<'a> {
    fn mark(&self, roots: impl Fn(&mut dyn FnMut(usize))) -> HashSet<usize> {
        let mut live: HashSet<usize> = HashSet::new();
        let mut stack: Vec<usize> = Vec::new();
        {
            let mut seed = |si: usize| {
                if si < self.index.symbols.len() && live.insert(si) {
                    stack.push(si);
                }
            };
            roots(&mut seed);
        }
        while let Some(si) = stack.pop() {
            let callees: Vec<usize> = self
                .callees_of
                .get(&si)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();
            for callee in callees {
                if live.insert(callee) {
                    stack.push(callee);
                }
            }
        }
        live
    }

    fn reachable(&self) -> (HashSet<usize>, HashSet<usize>) {
        let real = self.mark(|seed| {
            for (si, s) in self.index.symbols.iter().enumerate() {
                if s.entry
                    || (s.exported && self.entry_points.contains(s.file.as_str()))
                    || is_test_file(&s.file)
                {
                    seed(si);
                }
            }
            for si in 0..self.index.symbols.len() {
                if self.index.raw_references(si).iter().any(|&(_, _, from)| from == u32::MAX) {
                    seed(si);
                }
            }
        });
        let live = self.mark(|seed| {
            for &si in &real {
                seed(si);
            }
            for (si, s) in self.index.symbols.iter().enumerate() {
                if s.exported {
                    seed(si);
                }
            }
        });
        (real, live)
    }

    pub fn dead_code(&self, subdir: Option<&str>) -> DeadCodeReport<'a> {
        let mut report = self.dead_code_report(subdir);
        let names: HashSet<String> = report
            .candidates
            .iter()
            .map(|candidate| reflective::simple_name(&candidate.symbol.name).to_string())
            .filter(|name| name.len() >= 4)
            .collect();
        let hits = reflective::hits(&self.index.root, &names);
        for candidate in &mut report.candidates {
            if let Some(found) = hits.get(reflective::simple_name(&candidate.symbol.name)) {
                candidate.reflective_hit = Some(found.clone());
                candidate.tier = Tier::Low;
            }
        }
        report
    }

    pub fn dead_code_report(&self, subdir: Option<&str>) -> DeadCodeReport<'a> {
        let sub = subdir.map(normalize);
        let in_scope = |file: &str| sub.as_ref().is_none_or(|s| file.starts_with(s.as_str()));
        let (real, live) = self.reachable();

        let mut candidates: Vec<DeadCandidate> = Vec::new();
        for (si, s) in self.index.symbols.iter().enumerate() {
            if !in_scope(&s.file) || is_test_file(&s.file) {
                continue;
            }
            if s.exported && self.entry_points.contains(s.file.as_str()) {
                continue;
            }
            let refs = self.index.raw_references(si).len();
            if s.kind == "method" {
                if live.contains(&si) {
                    continue;
                }
                candidates.push(DeadCandidate {
                    symbol: s,
                    tier: Tier::Low,
                    reason: "method unreferenced statically — interfaces/dynamic dispatch may still call it; verify",
                    reflective_hit: None,
                });
            } else if s.exported {
                if real.contains(&si) {
                    continue;
                }
                candidates.push(DeadCandidate {
                    symbol: s,
                    tier: Tier::Low,
                    reason: if refs == 0 {
                        "exported but never referenced in-project — safe only if it isn't public API"
                    } else {
                        "exported and reached only from other dead code — verify it isn't public API"
                    },
                    reflective_hit: None,
                });
            } else {
                if live.contains(&si) {
                    continue;
                }
                candidates.push(DeadCandidate {
                    symbol: s,
                    tier: if refs == 0 { Tier::High } else { Tier::Medium },
                    reason: if refs == 0 {
                        "internal symbol with no references anywhere"
                    } else {
                        "internal, reached only from other unreachable code (dead island)"
                    },
                    reflective_hit: None,
                });
            }
        }
        candidates.sort_by(|a, b| {
            a.tier.cmp(&b.tier).then_with(|| {
                compare_paths(a.symbol.file.as_str(), b.symbol.file.as_str())
                    .then_with(|| a.symbol.line.cmp(&b.symbol.line))
            })
        });

        let mut files: Vec<&str> = Vec::new();
        for (file, ids) in &self.by_file {
            if !in_scope(file) || is_test_file(file) || self.entry_points.contains(file) {
                continue;
            }
            if self.imported.contains(file) {
                continue;
            }
            if !ids.is_empty() && ids.iter().all(|i| !live.contains(i)) {
                files.push(file);
            }
        }
        files.sort_by(|a, b| compare_paths(a, b));

        DeadCodeReport { candidates, files }
    }
}
