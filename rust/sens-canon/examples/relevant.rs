use std::path::Path;

use sens_canon::relevant::Catalog;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [root, prompt] = args.as_slice() else {
        eprintln!("relevant <carpeta> <petición>");
        std::process::exit(2);
    };
    let index = sens_index::build::build(Path::new(root));
    let catalog = Catalog::of(&index);
    for suggestion in catalog.relevant(prompt) {
        let symbol = &index.symbols[suggestion.symbol];
        println!("{:.2} {} {}:{} · {} usos", suggestion.score, symbol.name, symbol.file, symbol.line, suggestion.uses);
    }
}
