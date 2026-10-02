use std::path::PathBuf;

use sens_index::build;
use sens_index::index::Index;
use sens_index::query::{Engine, Tier};

const SAMPLES: [(&str, &str, &[&str]); 19] = [
    ("Shelf.swift", "class Shelf {\n    func size() -> Int {\n        return 1\n    }\n}\n\nfunc helper() -> Int {\n    return 2\n}\n", &["Shelf:class@1-5", "Shelf.size:method@2-4", "helper:function@7-9"]),
    ("shelf.dart", "class Shelf {\n  int size() {\n    return 1;\n  }\n}\n\nint helper() {\n  return 2;\n}\n", &["Shelf:class@1-5", "Shelf.size:method@2-4", "helper:function@7-9"]),
    ("Shelf.scala", "class Shelf {\n  def size(): Int = 1\n}\n\nobject Tools {\n  def helper(): Int = 2\n}\n", &["Shelf:class@1-3", "Shelf.size:method@2-2", "Tools:class@5-7", "Tools.helper:method@6-6"]),
    ("shelf.lua", "local Shelf = {}\n\nfunction Shelf.size()\n  return 1\nend\n\nlocal function helper()\n  return 2\nend\n", &["size:function@3-5", "helper:function@7-9"]),
    ("shelf.sh", "helper() {\n  echo 2\n}\n\nfunction size {\n  echo 1\n}\n", &["helper:function@1-3", "size:function@5-7"]),
    ("Shelf.ps1", "function Get-Size {\n  return 1\n}\n\nclass Shelf {\n  [int] Size() {\n    return 1\n  }\n}\n", &["Get-Size:function@1-3", "Shelf:class@5-9", "Shelf.Size:method@6-8"]),
    ("shelf.ex", "defmodule Shelf do\n  def size do\n    1\n  end\n\n  defp helper do\n    2\n  end\nend\n", &["Shelf:class@1-9", "Shelf.size:function@2-4", "Shelf.helper:function@6-8"]),
    ("Shelf.hs", "module Shelf where\n\nhelper :: Int -> Int\nhelper x = x + 2\n", &["helper:function@4-4"]),
    ("shelf.zig", "const Shelf = struct {\n    pub fn size(self: Shelf) u32 {\n        return 1;\n    }\n};\n\nfn helper() u32 {\n    return 2;\n}\n", &["size:function@2-4", "helper:function@7-9"]),
    ("shelf.R", "size <- function() {\n  1\n}\n\nhelper <- function(x) {\n  x + 2\n}\n", &["size:function@1-3", "helper:function@5-7"]),
    ("Shelf.m", "@implementation Shelf\n- (int)size {\n  return 1;\n}\n@end\n\nint helper(void) {\n  return 2;\n}\n", &["Shelf:class@1-5", "Shelf.size:method@2-4", "helper:function@7-9"]),
    ("shelf.ml", "let size () = 1\n\nlet helper x = x + 2\n\nmodule Shelf = struct\n  let count () = 3\nend\n", &["size:function@1-1", "helper:function@3-3", "Shelf:class@5-7", "Shelf.count:function@6-6"]),
    ("Shelf.fs", "module Shelf\n\nlet size () = 1\n\nlet helper x = x + 2\n", &["Shelf:class@1-5", "Shelf.size:function@3-3", "Shelf.helper:function@5-5"]),
    ("shelf.erl", "-module(shelf).\n-export([size/0]).\n\nsize() -> 1.\n\nhelper(X) -> X + 2.\n", &["size:function@4-4", "helper:function@6-6"]),
    ("shelf.jl", "function size()\n    return 1\nend\n\nstruct Shelf\n    count::Int\nend\n", &["size:function@1-3", "Shelf:class@5-7"]),
    ("Shelf.sol", "contract Shelf {\n    function size() public pure returns (uint) {\n        return 1;\n    }\n}\n", &["Shelf:class@1-5", "Shelf.size:method@2-4"]),
    ("Shelf.elm", "module Shelf exposing (size)\n\nsize : Int\nsize =\n    1\n\nhelper : Int -> Int\nhelper x =\n    x + 2\n", &["Shelf:class@1-1", "size:function@4-5", "helper:function@8-9"]),
    ("shelf.gleam", "pub fn size() -> Int {\n  1\n}\n\nfn helper(x: Int) -> Int {\n  x + 2\n}\n", &["size:function@1-3", "helper:function@5-7"]),
    ("Shelf.groovy", "class Shelf {\n    int size() {\n        return 1\n    }\n}\n\ndef helper() {\n    return 2\n}\n", &["Shelf:class@1-5", "Shelf.size:method@2-4", "helper:function@7-9"]),
];

const SWIFT_TOTALS: &str = "func totals(rows: [Row]) -> (Int, Int) {\n    var sum = 0\n    var count = 0\n    for row in rows {\n        if row.active {\n            sum += row.amount\n            count += 1\n        }\n    }\n    return (sum, count)\n}\n";

const ELIXIR_TOTALS: &str = "defmodule Totals do\n  def totals(rows) do\n    active = Enum.filter(rows, fn row -> row.active end)\n    sum = Enum.reduce(active, 0, fn row, acc -> acc + row.amount end)\n    count = Enum.count(active)\n    %{sum: sum, count: count, mean: if(count > 0, do: sum / count, else: 0)}\n  end\nend\n";

const BASH_BACKUP: &str = "backup() {\n  local source=\"$1\"\n  local target=\"$2\"\n  mkdir -p \"$target\"\n  tar -czf \"$target/$(date +%F).tar.gz\" -C \"$source\" .\n  find \"$target\" -name '*.tar.gz' -mtime +30 -delete\n  echo \"saved $source in $target\"\n}\n";

fn project(name: &str, files: &[(&str, &str)]) -> Index {
    let root = std::env::temp_dir().join("sens-index-languages").join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, content) in files {
        let file: PathBuf = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    build::build(&root)
}

#[test]
fn every_new_language_names_its_functions_and_classes() {
    let files: Vec<(&str, &str)> = SAMPLES.iter().map(|&(file, source, _)| (file, source)).collect();
    let index = project("names", &files);
    for (file, _, expected) in SAMPLES {
        let found: Vec<String> = index.symbols.iter().filter(|symbol| symbol.file == file).map(|symbol| format!("{}:{}@{}-{}", symbol.name, symbol.kind, symbol.line, symbol.end_line)).collect();
        assert_eq!(found, expected, "{file}");
    }
}

#[test]
fn a_renamed_copy_is_found_in_the_new_languages() {
    let index = project("copies", &[("lib/Totals.swift", SWIFT_TOTALS), ("lib/totals.ex", ELIXIR_TOTALS), ("bin/backup.sh", BASH_BACKUP)]);
    let proposals = [
        ("app/Report.swift", SWIFT_TOTALS.replace("totals", "summarize").replace("row", "entry"), "lib/Totals.swift#totals#1"),
        ("app/report.ex", ELIXIR_TOTALS.replace("Totals", "Report").replace("totals", "summarize").replace("row", "entry"), "lib/totals.ex#Totals.totals#2"),
        ("bin/save.sh", BASH_BACKUP.replace("backup()", "save()"), "bin/backup.sh#backup#1"),
    ];
    for (path, proposed, original) in proposals {
        let units = build::analyze(path, &proposed);
        let callable = units.iter().filter(|unit| unit.callable).max_by_key(|unit| unit.print.tokens).expect("a unit in the proposal");
        let found: Vec<String> = index.similar(&callable.print).into_iter().map(|found| index.units[found.unit].symbol.clone()).collect();
        assert_eq!(found.first().map(String::as_str), Some(original), "{path}: {found:?}");
    }
}

#[test]
fn an_unused_function_in_a_new_language_is_never_more_than_a_note() {
    let index = project("unused", &[("lib/Totals.swift", SWIFT_TOTALS), ("bin/backup.sh", BASH_BACKUP)]);
    let engine = Engine::new(&index, &index.entry_points);
    let report = engine.dead_code_report(None);
    assert!(!report.candidates.is_empty());
    assert!(report.candidates.iter().all(|candidate| candidate.tier == Tier::Low));
}

const PRIVATE: [(&str, &str, &str); 8] = [
    ("Shelf.swift", "func run() -> Int {\n    return used()\n}\n\nprivate func used() -> Int {\n    return 1\n}\n\nfileprivate func unused() -> Int {\n    return 2\n}\n", "unused"),
    ("shelf.dart", "int run() {\n  return _used();\n}\n\nint _used() {\n  return 1;\n}\n\nint _unused() {\n  return 2;\n}\n", "_unused"),
    ("shelf.lua", "local function used()\n  return 1\nend\n\nlocal function unused()\n  return 2\nend\n\nfunction run()\n  return used()\nend\n", "unused"),
    ("shelf.ex", "defmodule Shelf do\n  def run, do: used()\n\n  defp used, do: 1\n\n  defp unused, do: 2\nend\n", "Shelf.unused"),
    ("shelf.gleam", "pub fn run() -> Int {\n  used()\n}\n\nfn used() -> Int {\n  1\n}\n\nfn unused() -> Int {\n  2\n}\n", "unused"),
    ("shelf.zig", "pub fn run() u32 {\n    return used();\n}\n\nfn used() u32 {\n    return 1;\n}\n\nfn unused() u32 {\n    return 2;\n}\n", "unused"),
    ("Shelf.scala", "def run(): Int = used()\n\nprivate def used(): Int = 1\n\nprivate def unused(): Int = 2\n", "unused"),
    ("Shelf.fs", "module Shelf\n\nlet private used () = 1\n\nlet private unused () = 2\n\nlet run () = used ()\n", "Shelf.unused"),
];

#[test]
fn an_unused_private_function_is_dead_where_the_language_says_what_is_private() {
    for (file, source, dead) in PRIVATE {
        let index = project(&format!("private-{file}"), &[(file, source)]);
        let engine = Engine::new(&index, &index.entry_points);
        let blocking: Vec<&str> = engine.dead_code_report(None).candidates.iter().filter(|candidate| candidate.tier != Tier::Low).map(|candidate| candidate.symbol.name.as_str()).collect();
        assert_eq!(blocking, [dead], "{file}");
    }
}
