use std::collections::BTreeMap;
use std::path::Path;

use sens_canon::relevant::Catalog;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [root, patch] = args.as_slice() else {
        eprintln!("like <carpeta> <diff>");
        std::process::exit(2);
    };
    let index = sens_index::build::build(Path::new(root));
    let catalog = Catalog::of(&index);
    let mut added: BTreeMap<String, String> = BTreeMap::new();
    let mut file = String::new();
    for line in std::fs::read_to_string(patch).expect("diff").lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            file = path.to_string();
        } else if let Some(code) = line.strip_prefix('+').filter(|_| !line.starts_with("+++")) {
            added.entry(file.clone()).or_default().push_str(&format!("{code}\n"));
        }
    }
    for (file, code) in added.iter().filter(|(file, _)| !sens_index::testfile::is_test_file(file)) {
        let found: Vec<String> = catalog.like_code(code, file).iter().take(5).map(|found| format!("{} ({:.1})", index.symbols[found.symbol].name, found.score)).collect();
        println!("{file}: {}", found.join(", "));
    }
}
