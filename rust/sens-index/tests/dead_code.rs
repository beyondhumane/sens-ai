use std::path::{Path, PathBuf};

use sens_index::build;
use sens_index::index::Index;
use sens_index::query::{Engine, Tier};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

struct Candidate {
    name: String,
    file: String,
    kind: String,
    exported: bool,
    tier: Tier,
}

fn candidates(name: &str) -> Vec<Candidate> {
    let index: Index = build::build(&fixture(name));
    let engine = Engine::new(&index, &index.entry_points);
    engine
        .dead_code_report(None)
        .candidates
        .iter()
        .map(|found| Candidate {
            name: found.symbol.name.clone(),
            file: found.symbol.file.clone(),
            kind: found.symbol.kind.clone(),
            exported: found.symbol.exported,
            tier: found.tier,
        })
        .collect()
}

fn named<'a>(found: &'a [Candidate], name: &str) -> Vec<&'a Candidate> {
    found.iter().filter(|candidate| candidate.name == name).collect()
}

fn in_file<'a>(found: &'a [Candidate], name: &str, file: &str) -> Option<&'a Candidate> {
    found.iter().find(|candidate| candidate.name == name && candidate.file == file)
}

fn alive(found: &[Candidate], names: &[&str]) {
    for name in names {
        assert!(named(found, name).is_empty(), "{name} was flagged as dead");
    }
}

fn only<'a>(found: &'a [Candidate], name: &str) -> &'a Candidate {
    let matches = named(found, name);
    assert_eq!(matches.len(), 1, "{name}: {} candidates", matches.len());
    matches[0]
}

#[test]
fn vue_and_svelte_components_count_what_their_templates_use() {
    let found = candidates("components");
    alive(&found, &["onClick", "increment", "formatDate", "mount"]);
    assert!(in_file(&found, "unusedHelper", "src/App.vue").is_some(), "{:?}", found.iter().map(|candidate| &candidate.name).collect::<Vec<_>>());
    assert!(in_file(&found, "forgotten", "src/Counter.svelte").is_some());
}

#[test]
fn go_flags_dead_functions_and_never_live_or_entry_code() {
    let found = candidates("godead");
    let unused = only(&found, "unusedHelper");
    assert!(unused.tier == Tier::High && !unused.exported);
    let exported = only(&found, "ExportedUnused");
    assert!(exported.tier == Tier::Low && exported.exported);
    alive(&found, &["main", "init", "setup", "usedHelper", "Thing.Greet", "Greet", "helper1"]);
    assert!(in_file(&found, "Run", "pkg1/api.go").is_none());
    assert!(in_file(&found, "Shared", "pkg1/api.go").is_none());
    assert!(in_file(&found, "Shared", "pkg2/api.go").is_some_and(|dead| dead.tier == Tier::Low));
}

#[test]
fn python_flags_a_private_helper_and_keeps_framework_routes() {
    let found = candidates("pythondead");
    let helper = only(&found, "_helper");
    assert!(helper.file == "util.py" && helper.tier == Tier::High);
    alive(&found, &["ping", "health", "shared", "main"]);
    let index = build::build(&fixture("pythondead"));
    let mut entries: Vec<&str> = index.symbols.iter().filter(|symbol| symbol.entry).map(|symbol| symbol.name.as_str()).collect();
    entries.sort();
    assert_eq!(entries, ["health", "ping"]);
    assert!(named(&found, "Service.unused_static").first().is_some_and(|dead| dead.kind == "method"));
}

#[test]
fn java_classifies_candidates_without_false_positives() {
    let found = candidates("javadead");
    let orphan = only(&found, "Greeter.unusedPrivate");
    assert!(orphan.kind == "method" && orphan.tier == Tier::Low);
    let widget = only(&found, "Widget");
    assert!(widget.kind == "class" && widget.tier == Tier::Low);
    alive(&found, &["Main.main", "Main", "Greeter", "Greeter.greet", "Greeter.format", "Calculator", "Calculator.add", "PrintTask.run", "NotificationService", "AppConfig", "AppConfig.greeterBean", "AppConfig.onEvent", "StringUtil", "StringUtil.shout"]);
    let index = build::build(&fixture("javadead"));
    assert!(index.imports.iter().any(|edge| edge.to == "com/app/util/StringUtil.java"));
}

#[test]
fn kotlin_scopes_twins_by_package_and_keeps_composables() {
    let found = candidates("kotlindead");
    let secret = only(&found, "secretHelper");
    assert!(!secret.exported && secret.tier == Tier::High);
    let public = only(&found, "publicUnused");
    assert!(public.exported && public.tier == Tier::Low);
    alive(&found, &["main", "User", "greet", "Greeting", "Store.save", "Store"]);
    assert!(in_file(&found, "compute", "com/app/Greet.kt").is_none());
    assert!(in_file(&found, "compute", "com/other/Other.kt").is_some_and(|dead| dead.tier == Tier::High));
}

#[test]
fn csharp_keeps_entry_points_controllers_interfaces_and_overrides() {
    let found = candidates("csharpdead");
    assert!(only(&found, "Greeter.UnusedSecret").tier == Tier::Low);
    let orphan = only(&found, "OrphanWidget");
    assert!(orphan.tier == Tier::Low && orphan.exported);
    alive(&found, &["Program.Main", "UsersController", "UsersController.GetAll", "UsersController.Create", "Circle.Area", "IShape.Area", "Animal.Speak", "Dog.Speak", "Greeter", "Greeter.Format"]);
}

#[test]
fn php_keeps_every_kind_of_use_alive() {
    let found = candidates("phpdead");
    let never = only(&found, "neverCalled");
    assert!(never.tier == Tier::Low && never.file == "app.php" && never.kind == "function");
    let secret = only(&found, "User.unusedSecret");
    assert!(secret.tier == Tier::Low && secret.kind == "method");
    alive(&found, &["User", "Person", "Named", "User.getName", "User.normalize", "Registry", "Registry.register", "Registry.log", "formatName"]);
}

#[test]
fn ruby_flags_an_unused_top_level_method_only() {
    let found = candidates("rubydead");
    let dead = only(&found, "unused_top_level");
    assert!(dead.file == "models.rb" && dead.tier == Tier::Low);
    alive(&found, &["Widget", "Base", "Greeting", "Widget.render", "Base.shared", "Greeting.hello", "used_top_level", "App"]);
}

#[test]
fn c_ranks_by_linkage() {
    let found = candidates("cdead");
    let internal = found.iter().find(|candidate| candidate.name == "static_unused" && candidate.file.ends_with("a.c")).unwrap();
    assert!(internal.kind == "function" && !internal.exported && internal.tier == Tier::High);
    let linkable = found.iter().find(|candidate| candidate.name == "orphan_func" && candidate.file.ends_with("a.c")).unwrap();
    assert!(linkable.exported && linkable.tier == Tier::Low);
    alive(&found, &["main", "used_func", "static_used"]);
}

#[test]
fn cpp_ranks_by_linkage_and_resolves_includes() {
    let found = candidates("cppdead");
    for name in ["staticUnused", "anonUnused"] {
        let internal = only(&found, name);
        assert!(internal.tier == Tier::High && !internal.exported, "{name}");
    }
    let public = named(&found, "publicUnused");
    assert!(!public.is_empty() && public.iter().all(|dead| dead.tier == Tier::Low && dead.exported));
    alive(&found, &["main", "Shape.area", "Circle.area", "Circle.render", "usedHelper", "maxOf"]);
    let index = build::build(&fixture("cppdead"));
    assert!(index.imports.iter().any(|edge| edge.from == "main.cpp" && edge.to == "util.hpp"));
}

#[test]
fn rust_keeps_entry_points_methods_and_scoped_twins() {
    let found = candidates("rustdead");
    assert!(in_file(&found, "private_unused", "main.rs").is_some_and(|dead| dead.tier == Tier::High));
    assert!(in_file(&found, "public_unused", "main.rs").is_some_and(|dead| dead.tier == Tier::Low));
    alive(&found, &["main", "ffi_entry", "it_works", "run", "via_use", "greet", "Widget.new", "Widget.value", "Widget.speak"]);
    assert!(only(&found, "Widget.secret").tier == Tier::Low);
    assert!(in_file(&found, "helper", "alpha.rs").is_none());
    assert!(in_file(&found, "helper", "beta.rs").is_some_and(|dead| dead.tier == Tier::High));
}
