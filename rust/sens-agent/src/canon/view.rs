use serde::Serialize;

use sens_canon::card;
use sens_index::map;
use sens_index::testfile::is_test_file;

use super::keeper::Project;

const EXPORTS: usize = 8;
pub const REACH: usize = 3;

#[derive(Debug, PartialEq, Serialize)]
pub struct Region {
    pub name: String,
    pub files: Vec<String>,
    pub doors: Vec<String>,
    pub uses: Vec<String>,
    pub exports: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Central {
    pub path: String,
    pub dependents: usize,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Stray {
    pub path: String,
    pub area: String,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Link {
    pub from: String,
    pub to: String,
    pub weight: usize,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Shown {
    pub regions: Vec<Region>,
    pub links: Vec<Link>,
    pub central: Vec<Central>,
    pub strays: Vec<Stray>,
    pub cycles: Vec<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Reached {
    pub path: String,
    pub area: String,
    pub steps: usize,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Reach {
    pub file: String,
    pub area: String,
    pub dependents: Vec<Reached>,
    pub tests: Vec<Reached>,
}

pub fn shown(project: &Project) -> Shown {
    let (index, map) = (&project.index, &project.map);
    let path = |file: usize| index.files[file].path.clone();
    let mut exported = card::exports(index, map);
    let mut regions: Vec<Region> = map
        .areas
        .iter()
        .enumerate()
        .filter(|(_, area)| !area.tests)
        .map(|(at, area)| Region {
            name: area.name.clone(),
            files: area.files.iter().map(|&file| path(file)).collect(),
            doors: area.doors.iter().map(|&file| path(file)).collect(),
            uses: area.uses.iter().map(|&(used, _)| map.areas[used].name.clone()).collect(),
            exports: std::mem::take(&mut exported[at]).into_iter().take(EXPORTS).collect(),
        })
        .collect();
    regions.sort_by(|a, b| b.files.len().cmp(&a.files.len()).then(a.name.cmp(&b.name)));
    let links = map
        .areas
        .iter()
        .filter(|area| !area.tests)
        .flat_map(|area| area.uses.iter().map(|&(used, weight)| Link { from: area.name.clone(), to: map.areas[used].name.clone(), weight }))
        .collect();
    Shown {
        regions,
        links,
        cycles: map::cycles(index),
        central: map.hubs.iter().map(|&(file, dependents)| Central { path: path(file), dependents }).collect(),
        strays: map.strays.iter().map(|&(file, area)| Stray { path: path(file), area: map.areas[area].name.clone() }).collect(),
    }
}

pub fn sorted(project: &Project, distances: impl IntoIterator<Item = (usize, usize)>) -> (Vec<Reached>, Vec<Reached>) {
    let (index, map) = (&project.index, &project.map);
    let (tests, dependents): (Vec<Reached>, Vec<Reached>) = distances
        .into_iter()
        .map(|(file, steps)| Reached { path: index.files[file].path.clone(), area: map.areas[map.area_of[file]].name.clone(), steps })
        .partition(|found| is_test_file(&found.path));
    (dependents.into_iter().filter(|found| found.steps <= REACH).collect(), tests)
}

pub fn reach(project: &Project, file: &str) -> Option<Reach> {
    let (index, map) = (&project.index, &project.map);
    let slot = map.slot(file)?;
    let (dependents, tests) = sorted(project, map.spread(&[slot], usize::MAX).into_iter().filter(|&(_, steps)| steps > 0));
    Some(Reach { file: index.files[slot].path.clone(), area: map.areas[map.area_of[slot]].name.clone(), dependents, tests })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(name: &str) -> Project {
        let root = std::env::temp_dir().join("sens-canon-view").join(name);
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in [
            ("src/core/money.ts", "export function cents(value: number) { return Math.round(value * 100); }\n"),
            ("src/core/tax.ts", "import { cents } from './money.ts';\nexport function taxed(value: number) { return cents(value * 1.21); }\n"),
            ("src/core/round.ts", "export function rounded(value: number) { return Math.round(value); }\n"),
            ("web/cart.ts", "import { taxed } from '../src/core/tax.ts';\nexport function cart(value: number) { return taxed(value); }\n"),
            ("web/page.ts", "import { cart } from './cart.ts';\nexport function page() { return cart(1); }\n"),
            ("web/badge.ts", "import { cents } from '../src/core/money.ts';\nexport function badge() { return cents(2); }\n"),
            ("test/tax.test.ts", "import { taxed } from '../src/core/tax.ts';\ntaxed(1);\n"),
        ] {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        Project::of(&root)
    }

    #[test]
    fn the_map_is_shown_by_regions_without_the_tests() {
        let shown = shown(&project("shown"));
        let names: Vec<&str> = shown.regions.iter().map(|region| region.name.as_str()).collect();
        assert_eq!(names, ["src", "web"]);
        let src = &shown.regions[0];
        assert_eq!(src.files, ["src/core/money.ts", "src/core/round.ts", "src/core/tax.ts"]);
        assert_eq!(src.doors, ["src/core/money.ts", "src/core/tax.ts"]);
        assert!(src.exports.contains(&"cents".to_string()));
        assert_eq!(shown.regions[1].uses, ["src"]);
        assert_eq!(shown.links.len(), 1);
        assert_eq!((shown.links[0].from.as_str(), shown.links[0].to.as_str()), ("web", "src"));
        assert!(shown.links[0].weight >= 2);
        assert!(shown.cycles.is_empty());
    }

    #[test]
    fn what_a_file_reaches_comes_by_steps_with_its_tests_apart() {
        let project = project("reach");
        let reach = reach(&project, "src/core/tax.ts").unwrap();
        assert_eq!((reach.file.as_str(), reach.area.as_str()), ("src/core/tax.ts", "src"));
        assert_eq!(reach.dependents, [Reached { path: "web/cart.ts".into(), area: "web".into(), steps: 1 }, Reached { path: "web/page.ts".into(), area: "web".into(), steps: 2 }]);
        assert_eq!(reach.tests, [Reached { path: "test/tax.test.ts".into(), area: "test".into(), steps: 1 }]);
        assert!(super::reach(&project, "nowhere.ts").is_none());
    }
}
