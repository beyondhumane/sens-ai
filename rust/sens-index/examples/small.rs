use std::path::PathBuf;

use sens_index::build;

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).expect("carpeta"));
    let index = build::build(&root);
    let small: Vec<usize> = (0..index.units.len()).filter(|&at| index.units[at].small() && !index.units[at].test).collect();
    println!("pequeñas: {}", small.len());
    let line = |at: usize| {
        let unit = &index.units[at];
        let source = std::fs::read_to_string(index.root.join(&unit.file)).unwrap_or_default();
        source.lines().nth(unit.start_line.saturating_sub(1) as usize).unwrap_or_default().trim().chars().take(120).collect::<String>()
    };
    let mut pairs = Vec::new();
    for &at in &small {
        for found in index.small_like(&index.units[at].print) {
            if found.unit > at && index.units[found.unit].file != index.units[at].file {
                pairs.push((found.similarity(), at, found.unit));
            }
        }
    }
    pairs.sort_by(|a, b| b.0.total_cmp(&a.0));
    println!("pares: {}", pairs.len());
    for (similarity, one, other) in pairs {
        let (a, b) = (&index.units[one], &index.units[other]);
        println!("{similarity:.2} [{}|{}]\n   {}:{}  {}\n   {}:{}  {}", a.print.tokens, b.print.tokens, a.file, a.name, line(one), b.file, b.name, line(other));
    }
}
