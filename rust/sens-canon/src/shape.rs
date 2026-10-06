use std::collections::BTreeSet;

use sens_index::index::Index;
use sens_index::map::{self, Map};
use serde::{Deserialize, Serialize};

use crate::verdict::{Finding, Rule, Severity};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub areas: BTreeSet<String>,
    pub couplings: BTreeSet<(String, String)>,
    pub cycles: Vec<BTreeSet<String>>,
}

fn drawn(index: &Index, map: &Map) -> Shape {
    Shape {
        areas: map.areas.iter().map(|area| area.name.clone()).collect(),
        couplings: map.areas.iter().flat_map(|area| area.uses.iter().map(|&used| (area.name.clone(), map.areas[used].name.clone()))).collect(),
        cycles: map::cycles(index).into_iter().map(|cycle| cycle.into_iter().collect()).collect(),
    }
}

pub fn shape(index: &Index) -> Shape {
    drawn(index, &Map::of(index))
}

fn closed(cycle: &BTreeSet<String>) -> Finding {
    let files: Vec<&str> = cycle.iter().map(String::as_str).collect();
    Finding {
        rule: Rule::R9,
        severity: Severity::Consider,
        file: files[0].to_string(),
        line: 1,
        message: format!(
            "This turn closes an import cycle: {} import each other. Modules in a cycle load in an order nobody chose, and one of them sees the other half-built. Move what they share into a file they both import, or turn one import around.",
            files.join(", ")
        ),
        target: None,
        key: format!("R9:{}", files.join("~")),
    }
}

fn coupled(map: &Map, index: &Index, from: &str, to: &str) -> Finding {
    let door = map.areas.iter().find(|area| area.name == to).and_then(|area| area.doors.first()).map(|&file| index.files[file].path.clone());
    let through = door.as_deref().map(|door| format!(" If it is not, the usual way in is {door}.")).unwrap_or_default();
    Finding {
        rule: Rule::R10,
        severity: Severity::Note,
        file: door.clone().unwrap_or_else(|| to.to_string()),
        line: 1,
        message: format!("`{from}` now depends on `{to}`, which it did not use before. If the two areas are meant to know each other, nothing to do.{through}"),
        target: None,
        key: format!("R10:{from}→{to}"),
    }
}

pub fn findings(index: &Index, before: &Shape) -> Vec<Finding> {
    let map = Map::of(index);
    let now = drawn(index, &map);
    let cycles = now.cycles.iter().filter(|cycle| !before.cycles.iter().any(|old| cycle.is_subset(old))).map(closed);
    let couplings = now
        .couplings
        .iter()
        .filter(|(from, to)| !before.couplings.contains(&(from.clone(), to.clone())) && before.areas.contains(from) && before.areas.contains(to))
        .map(|(from, to)| coupled(&map, index, from, to));
    cycles.chain(couplings).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::build;

    fn project(name: &str, files: &[(&str, &str)]) -> Index {
        let root = std::env::temp_dir().join("sens-canon-shape").join(name);
        let _ = std::fs::remove_dir_all(&root);
        for (path, content) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, content).unwrap();
        }
        build::build(&root)
    }

    const MONEY: (&str, &str) = ("core/money.ts", "export const cents = (value: number) => Math.round(value * 100);\n");
    const TAX: (&str, &str) = ("core/tax.ts", "import { cents } from './money.ts';\nexport const taxed = (value: number) => cents(value * 1.21);\n");
    const CART: (&str, &str) = ("ui/cart.ts", "export const cart = (value: number) => value;\n");
    const DB: (&str, &str) = ("db/rows.ts", "export const rows = () => [];\n");

    #[test]
    fn a_new_cycle_is_held_once_and_an_old_one_is_not() {
        let before = shape(&project("cycle-before", &[MONEY, TAX, CART, DB]));
        assert!(before.cycles.is_empty());
        let looped = ("core/money.ts", "import { taxed } from './tax.ts';\nexport const cents = (value: number) => Math.round(value * 100);\nexport const back = () => taxed(1);\n");
        let after = project("cycle-after", &[looped, TAX, CART, DB]);
        let found = findings(&after, &before);
        assert_eq!(found.len(), 1);
        assert_eq!((found[0].rule, found[0].severity, found[0].key.as_str()), (Rule::R9, Severity::Consider, "R9:core/money.ts~core/tax.ts"));
        assert!(found[0].message.contains("core/money.ts, core/tax.ts import each other"), "{}", found[0].message);
        assert!(findings(&after, &shape(&after)).is_empty());
    }

    #[test]
    fn a_new_link_between_areas_is_noted_with_the_door_and_a_new_area_is_not() {
        let before = shape(&project("couple-before", &[MONEY, TAX, CART, DB]));
        let reaching = ("ui/cart.ts", "import { taxed } from '../core/tax.ts';\nexport const cart = (value: number) => taxed(value);\n");
        let fresh = ("fresh/new.ts", "import { rows } from '../db/rows.ts';\nexport const fresh = () => rows();\n");
        let after = project("couple-after", &[MONEY, TAX, reaching, DB, fresh]);
        let found = findings(&after, &before);
        let keys: Vec<(&str, Severity)> = found.iter().map(|finding| (finding.key.as_str(), finding.severity)).collect();
        assert_eq!(keys, [("R10:ui→core", Severity::Note)]);
        assert!(found[0].message.contains("the usual way in is core/tax.ts"), "{}", found[0].message);
    }
}
