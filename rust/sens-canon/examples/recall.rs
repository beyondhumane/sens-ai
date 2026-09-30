use std::path::Path;

use sens_canon::relevant::Catalog;
use sens_index::index::Index;

const SHOWN: usize = 8;

fn wanted(index: &Index, symbol: usize, expect: &[String]) -> bool {
    let found = &index.symbols[symbol];
    expect.iter().any(|one| one.rsplit_once(':').is_some_and(|(file, name)| found.name == name && found.file.ends_with(file)))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [root, calibration] = args.as_slice() else {
        eprintln!("recall <carpeta> <calibración.toml>");
        std::process::exit(2);
    };
    let table: toml::Table = std::fs::read_to_string(calibration).expect("calibración").parse().expect("toml");
    let index = sens_index::build::build(Path::new(root));
    let catalog = Catalog::of(&index);
    let queries = table["query"].as_array().expect("query");
    let (mut hits, mut reciprocal) = (0, 0.0);
    for query in queries {
        let text = query["text"].as_str().unwrap_or_default();
        let expect: Vec<String> = query["expect"].as_array().into_iter().flatten().filter_map(|one| one.as_str().map(String::from)).collect();
        let found = catalog.relevant(text);
        let rank = found.iter().take(SHOWN).position(|suggestion| wanted(&index, suggestion.symbol, &expect));
        if let Some(rank) = rank {
            hits += 1;
            reciprocal += 1.0 / (rank + 1) as f32;
        }
        let top: Vec<String> = found.iter().take(3).map(|suggestion| index.symbols[suggestion.symbol].name.clone()).collect();
        println!("{} {} · {}", rank.map_or("  -".into(), |rank| format!("{:>3}", rank + 1)), text, top.join(", "));
    }
    println!("\nacierto@{SHOWN}: {hits}/{} · MRR {:.3}", queries.len(), reciprocal / queries.len() as f32);
}
