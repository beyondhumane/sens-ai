use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use sens_index::build;
use sens_index::fingerprint::{self, MIN_TOKENS, UNIT_KINDS};
use sens_index::index::Index;

const THRESHOLDS: [f32; 6] = [0.6, 0.65, 0.7, 0.75, 0.8, 0.9];
const SAMPLE: usize = 400;
const SEED: u64 = 0x5E45_CA11;
const FLOOR: f32 = 0.5;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        fingerprint::mix(self.0)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound.max(1) as u64) as usize
    }
}

struct Sample {
    unit: usize,
    tokens: Vec<String>,
}

fn samples(index: &Index) -> Vec<Sample> {
    let mut found = Vec::new();
    for file in &index.files {
        let Ok(source) = std::fs::read_to_string(index.root.join(&file.path)) else {
            continue;
        };
        let Some((tree, emitted)) = build::parse(&file.path, &source) else {
            continue;
        };
        for symbol in emitted.symbols.iter().filter(|symbol| UNIT_KINDS.contains(&symbol.kind)) {
            let id = format!("{}#{}#{}", file.path, symbol.name, symbol.line);
            let Some(unit) = index.units.iter().position(|unit| unit.whole && unit.symbol == id && unit.comparable() && !unit.test) else {
                continue;
            };
            let Some(node) = tree.root_node().descendant_for_byte_range(symbol.start, symbol.end) else {
                continue;
            };
            let tokens = fingerprint::normalized(node, &source, fingerprint::own_name(&source, symbol.name_start));
            if tokens.len() >= MIN_TOKENS {
                found.push(Sample { unit, tokens });
            }
        }
    }
    found
}

fn slice(rng: &mut Rng, pool: &[Sample], avoid: usize) -> Vec<String> {
    let donor = loop {
        let at = rng.below(pool.len());
        if pool.len() == 1 || at != avoid {
            break &pool[at].tokens;
        }
    };
    let length = 5 + rng.below(8);
    let start = rng.below(donor.len().saturating_sub(length));
    donor[start..(start + length).min(donor.len())].to_vec()
}

fn mutate(rng: &mut Rng, pool: &[Sample], at: usize, inserts: usize, deletes: usize) -> Vec<String> {
    let mut tokens = pool[at].tokens.clone();
    for _ in 0..deletes {
        let length = (5 + rng.below(8)).min(tokens.len() / 4);
        let start = rng.below(tokens.len().saturating_sub(length));
        tokens.drain(start..start + length);
    }
    for _ in 0..inserts {
        let piece = slice(rng, pool, at);
        let position = rng.below(tokens.len());
        tokens.splice(position..position, piece);
    }
    tokens
}

fn main() {
    let roots: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    let mut out = String::new();
    for root in roots {
        let index = build::build(&root);
        let comparable: Vec<usize> = (0..index.units.len()).filter(|&at| index.units[at].whole && index.units[at].comparable() && !index.units[at].test).collect();

        let mut pairs: BTreeMap<(String, String), (f32, usize)> = BTreeMap::new();
        for &at in &comparable {
            let unit = &index.units[at];
            for found in index.similar_at(&unit.print, FLOOR) {
                let other = &index.units[found.unit];
                if !other.whole || other.symbol == unit.symbol || other.test {
                    continue;
                }
                let key = if unit.symbol < other.symbol { (unit.symbol.clone(), other.symbol.clone()) } else { (other.symbol.clone(), unit.symbol.clone()) };
                let share = found.similarity();
                let smaller = unit.print.tokens.min(other.print.tokens);
                pairs.entry(key).and_modify(|kept| kept.0 = kept.0.max(share)).or_insert((share, smaller));
            }
        }

        let pool = samples(&index);
        let mut rng = Rng(SEED);
        let picked: Vec<usize> = {
            let mut order: Vec<usize> = (0..pool.len()).collect();
            for at in (1..order.len()).rev() {
                order.swap(at, rng.below(at + 1));
            }
            order.into_iter().take(SAMPLE).collect()
        };
        let edits: [(&str, usize, usize); 5] = [("+1", 1, 0), ("+2", 2, 0), ("+3", 3, 0), ("-1", 0, 1), ("-1+1", 1, 1)];
        let mut recall: Vec<(&str, Vec<f32>)> = Vec::new();
        for (label, inserts, deletes) in edits {
            let found: Vec<f32> = picked
                .iter()
                .map(|&at| {
                    let tokens = mutate(&mut rng, &pool, at, inserts, deletes);
                    let print = fingerprint::print_of_tokens(0, &tokens);
                    index.similar_at(&print, FLOOR).iter().find(|found| found.unit == pool[at].unit).map_or(0.0, |found| found.similarity())
                })
                .collect();
            recall.push((label, found));
        }

        let _ = writeln!(out, "## {}\n", root.display());
        let _ = writeln!(out, "{} files, {} comparable functions, {} sampled for synthetic edits.\n", index.files.len(), comparable.len(), picked.len());
        let _ = writeln!(out, "| Threshold | Pairs of distinct functions at or above | {} |", recall.iter().map(|(label, _)| format!("Found after {label}")).collect::<Vec<_>>().join(" | "));
        let _ = writeln!(out, "| --- | --- | {} |", recall.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
        for threshold in THRESHOLDS {
            let flagged = pairs.values().filter(|(share, _)| *share >= threshold).count();
            let rates: Vec<String> = recall.iter().map(|(_, found)| format!("{:.0} %", 100.0 * found.iter().filter(|share| **share >= threshold).count() as f32 / found.len().max(1) as f32)).collect();
            let _ = writeln!(out, "| {threshold:.2} | {flagged} | {} |", rates.join(" | "));
        }
        let _ = writeln!(out, "\n| Smaller function, tokens | Pairs at or above {:.2} |\n| --- | --- |", fingerprint::NEAR);
        for (low, high) in [(30, 50), (50, 80), (80, 120), (120, usize::MAX)] {
            let count = pairs.values().filter(|(share, size)| *share >= fingerprint::NEAR && (low..high).contains(size)).count();
            let _ = writeln!(out, "| {low}–{} | {count} |", if high == usize::MAX { "…".to_string() } else { (high - 1).to_string() });
        }
        let _ = writeln!(out, "\nPairs at or above {:.2}, highest first:\n", fingerprint::NEAR);
        let mut ranked: Vec<_> = pairs.iter().filter(|(_, (share, _))| *share >= fingerprint::NEAR).collect();
        ranked.sort_by(|a, b| b.1.0.total_cmp(&a.1.0).then(a.0.cmp(b.0)));
        for ((left, right), (share, size)) in ranked {
            let _ = writeln!(out, "- {share:.2} · {size} tokens · `{left}` ~ `{right}`");
        }
        let _ = writeln!(out);
    }
    print!("{out}");
}
