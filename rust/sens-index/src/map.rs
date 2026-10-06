use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use crate::index::Index;
use crate::testfile::is_test_file;

const AREA_LIMIT: usize = 40;
const AREA_FLOOR: usize = 3;
const DOORS: usize = 2;
const HUBS: usize = 8;
const HUB_FLOOR: usize = 3;
const STRAY_FLOOR: usize = 3;
const CYCLE_LANGUAGES: [&str; 2] = ["typescript", "python"];
const ROOT: &str = ".";

#[derive(Debug, Default, PartialEq)]
pub struct Area {
    pub name: String,
    pub files: Vec<usize>,
    pub doors: Vec<usize>,
    pub uses: Vec<(usize, usize)>,
    pub tests: bool,
}

#[derive(Debug, Default)]
pub struct Map {
    pub areas: Vec<Area>,
    pub area_of: Vec<usize>,
    pub hubs: Vec<(usize, usize)>,
    pub strays: Vec<(usize, usize)>,
    dependents: Vec<BTreeSet<usize>>,
    slots: HashMap<String, usize>,
}

fn links(index: &Index) -> Vec<BTreeMap<usize, usize>> {
    let slot: HashMap<&str, usize> = index.files.iter().enumerate().map(|(at, file)| (file.path.as_str(), at)).collect();
    let mut links: Vec<BTreeMap<usize, usize>> = vec![BTreeMap::new(); index.files.len()];
    let mut link = |from: usize, to: usize| {
        if from != to {
            *links[from].entry(to).or_default() += 1;
        }
    };
    for edge in &index.imports {
        if let (Some(&from), Some(&to)) = (slot.get(edge.from.as_str()), slot.get(edge.to.as_str())) {
            link(from, to);
        }
    }
    let mut named: HashMap<&str, usize> = HashMap::new();
    for symbol in &index.symbols {
        *named.entry(symbol.name.as_str()).or_default() += 1;
    }
    for (at, symbol) in index.symbols.iter().enumerate().filter(|(_, symbol)| symbol.exported && named[symbol.name.as_str()] == 1) {
        let Some(&to) = slot.get(symbol.file.as_str()) else {
            continue;
        };
        for &(from, _, _) in index.raw_references(at) {
            if index.files[from as usize].language == index.files[to].language {
                link(from as usize, to);
            }
        }
    }
    links
}

fn joined(prefix: &str, name: &str) -> String {
    if prefix.is_empty() { name.to_string() } else { format!("{prefix}/{name}") }
}

fn carve<'a>(files: Vec<(usize, &'a str)>, prefix: &str, out: &mut Vec<(String, Vec<usize>)>) {
    let depth = if prefix.is_empty() { 0 } else { prefix.matches('/').count() + 1 };
    let mut own: Vec<usize> = Vec::new();
    let mut nested: BTreeMap<&'a str, Vec<(usize, &'a str)>> = BTreeMap::new();
    for (at, path) in files {
        match path.split('/').nth(depth).filter(|_| path.split('/').count() > depth + 1) {
            Some(folder) => nested.entry(folder).or_default().push((at, path)),
            None => own.push(at),
        }
    }
    let total = own.len() + nested.values().map(Vec::len).sum::<usize>();
    let name = if prefix.is_empty() { ROOT.to_string() } else { prefix.to_string() };
    if nested.is_empty() || (!prefix.is_empty() && total <= AREA_LIMIT) {
        own.extend(nested.into_values().flatten().map(|(at, _)| at));
        out.push((name, own));
        return;
    }
    for (folder, inner) in nested {
        if inner.len() < AREA_FLOOR && !prefix.is_empty() {
            own.extend(inner.into_iter().map(|(at, _)| at));
        } else {
            carve(inner, &joined(prefix, folder), out);
        }
    }
    if !own.is_empty() {
        out.push((name, own));
    }
}

fn ranked(counts: HashMap<usize, usize>, floor: usize, take: usize) -> Vec<(usize, usize)> {
    let mut ranked: Vec<(usize, usize)> = counts.into_iter().filter(|&(_, count)| count >= floor).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.truncate(take);
    ranked
}

impl Map {
    pub fn of(index: &Index) -> Map {
        let tested: Vec<bool> = index.files.iter().map(|file| is_test_file(&file.path)).collect();
        let mut carved = Vec::new();
        carve(index.files.iter().enumerate().map(|(at, file)| (at, file.path.as_str())).collect(), "", &mut carved);
        carved.sort();
        let mut area_of = vec![0; index.files.len()];
        let mut areas: Vec<Area> = carved
            .into_iter()
            .enumerate()
            .map(|(at, (name, mut files))| {
                files.sort();
                files.iter().for_each(|&file| area_of[file] = at);
                Area { tests: files.iter().all(|&file| tested[file]), name, files, ..Area::default() }
            })
            .collect();

        let links = links(index);
        let mut dependents: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); index.files.len()];
        let mut outsiders: HashMap<usize, BTreeSet<usize>> = HashMap::new();
        let mut leaning: Vec<HashMap<usize, (usize, usize)>> = vec![HashMap::new(); index.files.len()];
        let mut uses: Vec<HashMap<usize, usize>> = vec![HashMap::new(); areas.len()];
        for (from, targets) in links.iter().enumerate() {
            for (&to, &weight) in targets {
                dependents[to].insert(from);
                if tested[from] || tested[to] {
                    continue;
                }
                leaning[from].entry(area_of[to]).or_default().0 += weight;
                leaning[to].entry(area_of[from]).or_default().1 += weight;
                if area_of[from] != area_of[to] {
                    outsiders.entry(to).or_default().insert(from);
                    *uses[area_of[from]].entry(area_of[to]).or_default() += weight;
                }
            }
        }

        for (at, area) in areas.iter_mut().enumerate() {
            let doors: HashMap<usize, usize> = area.files.iter().filter_map(|file| outsiders.get(file).map(|from| (*file, from.len()))).collect();
            area.doors = ranked(doors, 1, DOORS).into_iter().map(|(file, _)| file).collect();
            area.uses = ranked(std::mem::take(&mut uses[at]), 1, usize::MAX);
        }
        let depended: HashMap<usize, usize> = (0..index.files.len()).filter(|&file| !tested[file]).map(|file| (file, dependents[file].iter().filter(|&&from| !tested[from]).count())).collect();
        let hubs = ranked(depended, HUB_FLOOR, HUBS);
        let strays = leaning
            .iter()
            .enumerate()
            .filter_map(|(file, weights)| {
                let own = weights.get(&area_of[file]).copied().unwrap_or_default();
                let (&other, &(uses, used)) = weights.iter().filter(|(area, _)| **area != area_of[file]).max_by(|a, b| (a.1.0 + a.1.1).cmp(&(b.1.0 + b.1.1)).then(b.0.cmp(a.0)))?;
                let pulled = uses > own.0 && used > own.1 && uses + used >= STRAY_FLOOR && uses + used >= 2 * (own.0 + own.1);
                pulled.then_some((file, other))
            })
            .collect();
        let slots = index.files.iter().enumerate().map(|(at, file)| (file.path.clone(), at)).collect();
        Map { areas, area_of, hubs, strays, dependents, slots }
    }

    pub fn slot(&self, path: &str) -> Option<usize> {
        let path = path.trim().replace('\\', "/");
        let path = path.trim_start_matches("./");
        let suffix = format!("/{path}");
        self.slots.get(path).copied().or_else(|| self.slots.iter().filter(|(known, _)| known.ends_with(&suffix)).map(|(_, &at)| at).min())
    }

    pub fn spread(&self, seeds: &[usize], depth: usize) -> Vec<(usize, usize)> {
        let mut distance: BTreeMap<usize, usize> = seeds.iter().map(|&seed| (seed, 0)).collect();
        let mut queue: VecDeque<usize> = seeds.iter().copied().collect();
        while let Some(file) = queue.pop_front() {
            let next = distance[&file] + 1;
            if next > depth {
                continue;
            }
            for &from in &self.dependents[file] {
                if let Entry::Vacant(slot) = distance.entry(from) {
                    slot.insert(next);
                    queue.push_back(from);
                }
            }
        }
        let mut reached: Vec<(usize, usize)> = distance.into_iter().collect();
        reached.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));
        reached
    }

    pub fn placement(&self, index: &Index) -> BTreeMap<String, String> {
        index.files.iter().enumerate().filter(|(_, file)| !is_test_file(&file.path)).map(|(at, file)| (file.path.clone(), self.areas[self.area_of[at]].name.clone())).collect()
    }
}

fn finished(out: &[Vec<usize>]) -> Vec<usize> {
    let mut seen = vec![false; out.len()];
    let mut order = Vec::with_capacity(out.len());
    for start in 0..out.len() {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut stack = vec![(start, 0)];
        while let Some(&(node, next)) = stack.last() {
            let top = stack.len() - 1;
            match out[node].get(next) {
                Some(&child) => {
                    stack[top].1 += 1;
                    if !seen[child] {
                        seen[child] = true;
                        stack.push((child, 0));
                    }
                }
                None => {
                    order.push(node);
                    stack.pop();
                }
            }
        }
    }
    order
}

pub fn cycles(index: &Index) -> Vec<Vec<String>> {
    let slot: HashMap<&str, usize> = index
        .files
        .iter()
        .enumerate()
        .filter(|(_, file)| CYCLE_LANGUAGES.contains(&file.language) && !is_test_file(&file.path))
        .map(|(at, file)| (file.path.as_str(), at))
        .collect();
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); index.files.len()];
    let mut into: Vec<Vec<usize>> = vec![Vec::new(); index.files.len()];
    for edge in &index.imports {
        if let (Some(&from), Some(&to)) = (slot.get(edge.from.as_str()), slot.get(edge.to.as_str()))
            && from != to
        {
            out[from].push(to);
            into[to].push(from);
        }
    }
    let mut group = vec![usize::MAX; index.files.len()];
    let mut found: Vec<Vec<String>> = Vec::new();
    for start in finished(&out).into_iter().rev() {
        if group[start] != usize::MAX {
            continue;
        }
        group[start] = start;
        let mut members = vec![start];
        let mut stack = vec![start];
        while let Some(node) = stack.pop() {
            for &from in &into[node] {
                if group[from] == usize::MAX {
                    group[from] = start;
                    members.push(from);
                    stack.push(from);
                }
            }
        }
        if members.len() > 1 {
            members.sort();
            found.push(members.into_iter().map(|file| index.files[file].path.clone()).collect());
        }
    }
    found.sort();
    found
}

pub fn moved(before: &BTreeMap<String, String>, after: &BTreeMap<String, String>) -> Vec<String> {
    let mut lines = Vec::new();
    for (file, area) in after {
        match before.get(file) {
            None => lines.push(format!("+ {file} ({area})")),
            Some(was) if was != area => lines.push(format!("~ {file}: {was} → {area}")),
            Some(_) => {}
        }
    }
    lines.extend(before.keys().filter(|file| !after.contains_key(*file)).map(|file| format!("- {file}")));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;
    use std::path::PathBuf;

    fn project(name: &str, files: &[(&str, &str)]) -> Index {
        let root: PathBuf = std::env::temp_dir().join("sens-map").join(name);
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        build::build(&root)
    }

    fn names(map: &Map) -> Vec<&str> {
        map.areas.iter().map(|area| area.name.as_str()).collect()
    }

    fn path(index: &Index, file: usize) -> &str {
        &index.files[file].path
    }

    fn shop(name: &str) -> Index {
        project(
            name,
            &[
                ("src/money/format.ts", "export function price(cents: number) { return `${cents / 100} €`; }\n"),
                ("src/money/tax.ts", "import { price } from './format.ts';\nexport function taxed(cents: number) { return price(cents * 1.21); }\n"),
                ("src/money/round.ts", "export function rounded(cents: number) { return Math.round(cents); }\n"),
                ("src/cart/total.ts", "import { taxed } from '../money/tax.ts';\nimport { price } from '../money/format.ts';\nexport function total(items: number[]) { return taxed(items.reduce((a, b) => a + b, 0)) + price(0); }\n"),
                ("src/cart/view.ts", "import { total } from './total.ts';\nimport { price } from '../money/format.ts';\nexport function view(items: number[]) { return `${total(items)} ${price(1)}`; }\n"),
                ("src/cart/badge.ts", "import { price } from '../money/format.ts';\nimport { taxed } from '../money/tax.ts';\nimport { rounded } from '../money/round.ts';\nexport function badge(a: number) { return price(a) + taxed(a) + price(rounded(a)) + taxed(rounded(a)); }\n"),
                ("src/main.ts", "import { view } from './cart/view.ts';\nview([1]);\n"),
                ("test/total.test.ts", "import { total } from '../src/cart/total.ts';\ntotal([1]);\n"),
            ],
        )
    }

    #[test]
    fn a_small_project_gets_one_area_per_top_folder() {
        let index = shop("areas");
        let map = Map::of(&index);
        assert_eq!(names(&map), ["src", "test"]);
        assert!(map.areas[1].tests && !map.areas[0].tests);
        assert_eq!(map.area_of.len(), index.files.len());
    }

    #[test]
    fn a_large_folder_is_cut_by_its_subfolders() {
        let mut files: Vec<(String, String)> = Vec::new();
        for at in 0..AREA_LIMIT {
            files.push((format!("app/ui/view{at}.ts"), format!("export const view{at} = {at};\n")));
        }
        for at in 0..5 {
            files.push((format!("app/data/store{at}.ts"), format!("export const store{at} = {at};\n")));
        }
        files.push(("app/tiny/one.ts".into(), "export const one = 1;\n".into()));
        files.push(("app/main.ts".into(), "export const main = 1;\n".into()));
        let borrowed: Vec<(&str, &str)> = files.iter().map(|(path, content)| (path.as_str(), content.as_str())).collect();
        let index = project("large", &borrowed);
        let map = Map::of(&index);
        assert_eq!(names(&map), ["app", "app/data", "app/ui"]);
        let app = &map.areas[0];
        assert_eq!(app.files.iter().map(|&file| path(&index, file)).collect::<Vec<_>>(), ["app/main.ts", "app/tiny/one.ts"]);
    }

    #[test]
    fn doors_hubs_uses_and_strays_come_from_the_graph() {
        let index = project(
            "doors",
            &[
                ("core/a.ts", "export function a() { return 1; }\n"),
                ("core/b.ts", "import { a } from './a.ts';\nexport function b() { return a(); }\n"),
                ("core/c.ts", "export function c() { return 3; }\n"),
                ("core/lost.ts", "import { x } from '../ui/x.ts';\nimport { y } from '../ui/y.ts';\nexport function lost() { return x() + y() + x(); }\n"),
                ("ui/x.ts", "import { a } from '../core/a.ts';\nexport function x() { return a(); }\n"),
                ("ui/y.ts", "import { a } from '../core/a.ts';\nimport { b } from '../core/b.ts';\nexport function y() { return a() + b(); }\n"),
                ("ui/z.ts", "import { a } from '../core/a.ts';\nimport { lost } from '../core/lost.ts';\nexport function z() { return a() + lost(); }\n"),
            ],
        );
        let map = Map::of(&index);
        assert_eq!(names(&map), ["core", "ui"]);
        let core = &map.areas[0];
        assert_eq!(path(&index, core.doors[0]), "core/a.ts");
        assert_eq!(core.uses.iter().map(|&(used, _)| used).collect::<Vec<_>>(), [1]);
        assert_eq!(map.areas[1].uses.iter().map(|&(used, _)| used).collect::<Vec<_>>(), [0]);
        assert_eq!(path(&index, map.hubs[0].0), "core/a.ts");
        assert_eq!(map.hubs[0].1, 4);
        assert!(map.strays.iter().any(|&(file, area)| path(&index, file) == "core/lost.ts" && area == 1), "{:?}", map.strays);
        assert!(!map.strays.iter().any(|&(file, _)| path(&index, file) == "core/a.ts"));
    }

    #[test]
    fn what_depends_on_a_file_spreads_by_distance() {
        let index = shop("spread");
        let map = Map::of(&index);
        let format = map.slot("src/money/format.ts").unwrap();
        let reached: Vec<(&str, usize)> = map.spread(&[format], usize::MAX).into_iter().map(|(file, distance)| (path(&index, file), distance)).collect();
        assert_eq!(reached[0], ("src/money/format.ts", 0));
        assert!(reached.contains(&("src/money/tax.ts", 1)));
        assert!(reached.contains(&("src/cart/view.ts", 1)));
        assert!(reached.contains(&("src/main.ts", 2)));
        assert!(reached.contains(&("test/total.test.ts", 2)));
        assert!(map.spread(&[format], 1).iter().all(|&(_, distance)| distance <= 1));
        assert_eq!(map.slot("money/format.ts"), Some(format));
        assert_eq!(map.slot("nowhere.ts"), None);
    }

    #[test]
    fn the_map_is_the_same_every_time() {
        let index = shop("same");
        let (one, two) = (Map::of(&index), Map::of(&index));
        assert_eq!(one.areas, two.areas);
        assert_eq!((one.hubs, one.strays), (two.hubs, two.strays));
    }

    #[test]
    fn import_cycles_are_found_where_they_hurt_and_tests_stay_out() {
        let index = project(
            "cycles",
            &[
                ("web/a.ts", "import { b } from './b.ts';\nexport const a = () => b();\n"),
                ("web/b.ts", "import { c } from './c.ts';\nexport const b = () => c();\n"),
                ("web/c.ts", "import { a } from './a.ts';\nexport const c = () => a();\n"),
                ("web/d.ts", "import { a } from './a.ts';\nexport const d = () => a();\n"),
                ("web/a.test.ts", "import { d } from './d.ts';\nimport { a } from './a.ts';\nexport const t = () => d() + a();\n"),
                ("crate/lib.rs", "mod one;\nmod two;\n"),
                ("crate/one.rs", "use crate::two::two;\npub fn one() { two() }\n"),
                ("crate/two.rs", "use crate::one::one;\npub fn two() { one() }\n"),
            ],
        );
        assert_eq!(cycles(&index), [vec!["web/a.ts".to_string(), "web/b.ts".into(), "web/c.ts".into()]]);
    }

    #[test]
    fn a_change_of_placement_reads_as_added_moved_and_removed() {
        let before: BTreeMap<String, String> = [("a.ts", "src"), ("b.ts", "src"), ("c.ts", "src")].into_iter().map(|(file, area)| (file.into(), area.into())).collect();
        let after: BTreeMap<String, String> = [("a.ts", "src"), ("b.ts", "lib"), ("d.ts", "lib")].into_iter().map(|(file, area)| (file.into(), area.into())).collect();
        assert_eq!(moved(&before, &after), ["~ b.ts: src → lib", "+ d.ts (lib)", "- c.ts"]);
        assert!(moved(&after, &after).is_empty());
    }
}
