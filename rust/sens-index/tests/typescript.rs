use std::path::{Path, PathBuf};

use sens_index::build;
use sens_index::index::Index;
use sens_index::query::{Engine, Tier};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

fn dead(name: &str) -> Vec<String> {
    let index = build::build(&fixture(name));
    let engine = Engine::new(&index, &index.entry_points);
    engine.dead_code(None).candidates.iter().map(|candidate| candidate.symbol.name.clone()).collect()
}

fn uses(index: &Index, name: &str) -> usize {
    let at = index.symbols.iter().position(|symbol| symbol.name == name).unwrap_or_else(|| panic!("no symbol {name}"));
    index.raw_references(at).len()
}

#[test]
fn symbols_references_and_callers_come_out_of_a_small_project() {
    let index = build::build(&fixture("sample"));
    let names: Vec<&str> = index.symbols.iter().map(|symbol| symbol.name.as_str()).collect();
    for expected in ["add", "subtract", "unusedHelper", "PI", "main"] {
        assert!(names.contains(&expected), "{expected} missing from {names:?}");
    }
    let add = index.symbols.iter().position(|symbol| symbol.name == "add").unwrap();
    let main = index.symbols.iter().position(|symbol| symbol.name == "main").unwrap();
    let sites = index.references_at(add);
    assert!(sites.iter().any(|site| site.file.ends_with("app.ts")), "{sites:?}");
    assert!(sites.iter().any(|site| site.from.as_deref() == Some(index.symbols[main].id.as_str())), "{sites:?}");
    assert_eq!(uses(&index, "subtract"), 0);
    assert_eq!(uses(&index, "unusedHelper"), 0);
}

#[test]
fn a_package_main_is_public_and_what_it_does_not_reach_is_dead() {
    let found = dead("pkgentry");
    assert!(!found.contains(&"publicThing".into()), "{found:?}");
    assert!(found.contains(&"privateThing".into()), "{found:?}");
}

#[test]
fn an_unreachable_module_is_reported_as_a_dead_file() {
    let index = build::build(&fixture("deadfile"));
    let engine = Engine::new(&index, &index.entry_points);
    let report = engine.dead_code(None);
    assert!(report.files.contains(&"orphan.ts"), "{:?}", report.files);
    assert!(!report.files.contains(&"main.ts"), "{:?}", report.files);
}

#[test]
fn a_name_found_in_a_config_file_is_kept_at_low_with_its_source() {
    let index = build::build(&fixture("reflective"));
    let engine = Engine::new(&index, &index.entry_points);
    let report = engine.dead_code(None);
    for name in ["reflectiveHandler", "internalHook"] {
        let found = report.candidates.iter().find(|candidate| candidate.symbol.name == name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(found.tier, Tier::Low, "{name}");
        assert!(found.reflective_hit.as_deref().is_some_and(|hit| hit.contains("config.json")), "{name}");
    }
}

#[test]
fn an_unused_method_is_low_and_a_used_one_is_alive() {
    let index = build::build(&fixture("methods"));
    let engine = Engine::new(&index, &index.entry_points);
    let report = engine.dead_code(None);
    assert!(report.candidates.iter().any(|candidate| candidate.symbol.name == "Widget.orphanMethod" && candidate.tier == Tier::Low));
    assert!(!report.candidates.iter().any(|candidate| candidate.symbol.name == "Widget.helper"));
}

#[test]
fn jsx_tags_and_type_only_uses_count_as_uses() {
    let jsx = dead("jsx");
    assert!(!jsx.contains(&"Button".into()) && !jsx.contains(&"Panel".into()), "{jsx:?}");
    let types = dead("typeonly");
    assert!(!types.contains(&"Shape".into()) && !types.contains(&"Handler".into()), "{types:?}");
}

#[test]
fn a_monorepo_package_entry_is_public() {
    let found = dead("monorepo");
    assert!(!found.contains(&"fooApi".into()), "{found:?}");
    assert!(found.contains(&"fooHelper".into()), "{found:?}");
}

#[test]
fn barrels_pass_named_and_default_exports_through() {
    let found = dead("barrel");
    assert!(!found.contains(&"barreled".into()) && !found.contains(&"Widget".into()), "{found:?}");
}

#[test]
fn a_path_alias_from_tsconfig_resolves() {
    let found = dead("alias");
    assert!(!found.contains(&"aliased".into()), "{found:?}");
}

#[test]
fn dynamic_imports_string_names_and_star_reexports_are_uses() {
    let index = build::build(&fixture("dynimport"));
    assert!(uses(&index, "lazyThing") > 0);
    assert!(uses(&index, "registered") > 0);
    assert!(uses(&index, "viaStar") > 0);
    assert_eq!(uses(&index, "trulyUnused"), 0);
}

#[test]
fn object_shorthand_is_a_use() {
    let index = build::build(&fixture("shorthand"));
    assert!(uses(&index, "alpha") > 0);
    let found = dead("shorthand");
    assert!(!found.contains(&"alpha".into()) && !found.contains(&"beta".into()), "{found:?}");
}
