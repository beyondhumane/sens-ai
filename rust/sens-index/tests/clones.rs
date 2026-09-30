use std::path::PathBuf;

use sens_index::build;
use sens_index::fingerprint::Likeness;
use sens_index::index::Index;

const TOTALS: &str = "export function totals(rows: Row[]) {
  let sum = 0;
  let count = 0;
  for (const row of rows) {
    if (row.active) {
      sum += row.amount;
      count += 1;
    }
  }
  return { sum, count, mean: count ? sum / count : 0 };
}
";

const VIEW: &str = "export function render(user: User) {
  const card = document.createElement('div');
  card.className = 'card';
  card.textContent = user.name;
  document.body.appendChild(card);
  return card;
}
";

const SLUG: &str = "import re\nimport unicodedata\n\n\ndef slugify(title):\n    plain = unicodedata.normalize('NFKD', title).encode('ascii', 'ignore').decode()\n    slug = re.sub(r'[^a-z0-9]+', '-', plain.lower()).strip('-')\n    return slug[:60].rstrip('-')\n";

fn project(name: &str) -> Index {
    let root = std::env::temp_dir().join("sens-index-clones").join(name);
    let _ = std::fs::remove_dir_all(&root);
    for (path, content) in [("src/lib/totals.ts", TOTALS), ("src/ui/view.ts", VIEW), ("blog/text.py", SLUG)] {
        let file: PathBuf = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, content).unwrap();
    }
    build::build(&root)
}

fn matches(index: &Index, path: &str, proposed: &str) -> Vec<(String, Likeness)> {
    let units = build::analyze(path, proposed);
    let whole = units.iter().find(|unit| unit.whole).expect("a unit in the proposal");
    index.similar(&whole.print).into_iter().map(|found| (index.units[found.unit].symbol.clone(), found.likeness)).collect()
}

#[test]
fn a_proposed_copy_with_other_names_points_at_the_original() {
    let index = project("renamed");
    let renamed = TOTALS.replace("totals", "summarize").replace("rows", "entries").replace("row", "entry").replace("sum", "total");
    let found = matches(&index, "src/report.ts", &renamed);
    assert_eq!(found.first(), Some(&("src/lib/totals.ts#totals#1".to_string(), Likeness::Renamed)), "{found:?}");
}

#[test]
fn a_proposed_near_copy_is_found_and_a_stranger_is_not() {
    let index = project("near");
    let near = TOTALS.replace("totals", "tally").replace("  return", "  console.log(sum);\n  return");
    let found = matches(&index, "src/report.ts", &near);
    assert!(matches!(found.first(), Some((symbol, Likeness::Near(_))) if symbol == "src/lib/totals.ts#totals#1"), "{found:?}");
    let stranger = "export function greet(user: User) {\n  const words = ['hello', user.first, user.last];\n  const line = words.filter(Boolean).join(' ');\n  window.alert(line);\n  history.pushState({}, '', '/greeted');\n  return line.length;\n}\n";
    assert!(matches(&index, "src/greet.ts", stranger).is_empty());
}

#[test]
fn copies_are_found_in_python_as_well() {
    let index = project("python");
    let copy = SLUG.replace("slugify", "make_slug").replace("title", "text").replace("plain", "folded");
    let found = matches(&index, "blog/other.py", &copy);
    assert_eq!(found.first().map(|(symbol, _)| symbol.as_str()), Some("blog/text.py#slugify#5"), "{found:?}");
}

#[test]
fn a_file_the_index_does_not_understand_has_no_units() {
    assert!(build::analyze("README.md", "# hello").is_empty());
}
