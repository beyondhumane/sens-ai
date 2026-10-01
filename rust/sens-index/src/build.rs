use std::collections::{HashMap, HashSet};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use crate::entries;
use crate::fingerprint::{self, Unit};
use crate::lang::treesitter::Emitted;
use crate::testfile::is_test_file;
use crate::index::Index;
use crate::lang::treesitter::{self, Extract, Grammar, Options, Prepare, as_is};
use crate::lang::{cfamily, csharp, go, java, kotlin, php, python, ruby, rust, typescript};

const SKIP_DIRS: [&str; 8] = ["node_modules", "dist", ".sens", ".git", "target", "__pycache__", ".venv", "venv"];

struct Language {
    name: &'static str,
    extensions: &'static [&'static str],
    grammar: Grammar,
    prepare: Prepare,
    extract: Extract,
    options: fn() -> Options,
}

const LANGUAGES: [Language; 11] = [
    Language { name: "typescript", extensions: &["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs", "vue", "svelte"], grammar: typescript::grammar, prepare: typescript::prepare, extract: typescript::extract, options: typescript::options },
    Language { name: "go", extensions: &["go"], grammar: |_| tree_sitter_go::LANGUAGE.into(), prepare: as_is, extract: go::extract, options: go::options },
    Language { name: "ruby", extensions: &["rb"], grammar: |_| tree_sitter_ruby::LANGUAGE.into(), prepare: as_is, extract: ruby::extract, options: ruby::options },
    Language { name: "php", extensions: &["php"], grammar: |_| tree_sitter_php::LANGUAGE_PHP.into(), prepare: as_is, extract: php::extract, options: php::options },
    Language { name: "java", extensions: &["java"], grammar: |_| tree_sitter_java::LANGUAGE.into(), prepare: as_is, extract: java::extract, options: java::options },
    Language { name: "csharp", extensions: &["cs"], grammar: |_| tree_sitter_c_sharp::LANGUAGE.into(), prepare: as_is, extract: csharp::extract, options: csharp::options },
    Language { name: "kotlin", extensions: &["kt", "kts"], grammar: |_| tree_sitter_kotlin_ng::LANGUAGE.into(), prepare: as_is, extract: kotlin::extract, options: kotlin::options },
    Language { name: "python", extensions: &["py", "pyi"], grammar: |_| tree_sitter_python::LANGUAGE.into(), prepare: as_is, extract: python::extract, options: python::options },
    Language { name: "rust", extensions: &["rs"], grammar: |_| tree_sitter_rust::LANGUAGE.into(), prepare: as_is, extract: rust::extract, options: rust::options },
    Language { name: "c", extensions: &["c"], grammar: |_| tree_sitter_c::LANGUAGE.into(), prepare: as_is, extract: cfamily::extract_c, options: cfamily::options },
    Language { name: "cpp", extensions: &["cpp", "cxx", "cc", "hpp", "hh", "hxx", "h"], grammar: |_| tree_sitter_cpp::LANGUAGE.into(), prepare: as_is, extract: cfamily::extract_cpp, options: cfamily::options },
];

fn language_of(path: &Path) -> Option<&'static Language> {
    let name = path.file_name()?.to_str()?;
    if name.ends_with(".d.ts") || name.contains(".min.") {
        return None;
    }
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    LANGUAGES.iter().find(|language| language.extensions.contains(&extension.as_str()))
}

pub fn indexable(path: &Path) -> bool {
    language_of(path).is_some()
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace(MAIN_SEPARATOR, "/")
}

pub fn collect(root: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = ignore::WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .require_git(false)
        .filter_entry(|entry| !SKIP_DIRS.contains(&entry.file_name().to_str().unwrap_or("")))
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(ignore::DirEntry::into_path)
        .filter(|path| indexable(path))
        .collect();
    found.sort();
    found
}

pub fn build(root: &Path) -> Index {
    let mut grouped: HashMap<&'static str, Vec<(String, PathBuf)>> = HashMap::new();
    for path in collect(root) {
        if let Some(language) = language_of(&path) {
            grouped.entry(language.name).or_default().push((relative(root, &path), path));
        }
    }

    let (mut files, mut symbols, mut imports, mut references, mut units) = (Vec::new(), Vec::new(), Vec::new(), HashMap::new(), Vec::new());
    for language in &LANGUAGES {
        let Some(found) = grouped.get(language.name) else {
            continue;
        };
        let contribution = treesitter::build(language.name, found, language.grammar, language.prepare, language.extract, (language.options)());
        symbols.extend(contribution.symbols);
        files.extend(contribution.files);
        imports.extend(contribution.imports);
        references.extend(contribution.references);
        units.extend(contribution.units);
    }
    files.sort_by(|a: &crate::index::FileInfo, b| a.path.cmp(&b.path));
    let mut index = Index::assemble(root.to_path_buf(), files, symbols, imports, references, units);
    index.entry_points = entries::find(root, &index);
    index
}

fn parsed(language: &Language, path: &str, source: &str) -> Option<(tree_sitter::Tree, Emitted)> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&(language.grammar)(path)).ok()?;
    let tree = parser.parse(source, None)?;
    let mut emitted = Emitted::default();
    (language.extract)(&tree.root_node(), source, path, &HashSet::new(), &mut emitted);
    Some((tree, emitted))
}

pub fn parse(path: &str, source: &str) -> Option<(tree_sitter::Tree, Emitted)> {
    let language = language_of(Path::new(path))?;
    parsed(language, path, &(language.prepare)(path, source.to_string()))
}

pub fn analyze(path: &str, source: &str) -> Vec<Unit> {
    let Some(language) = language_of(Path::new(path)) else {
        return Vec::new();
    };
    let source = (language.prepare)(path, source.to_string());
    parsed(language, path, &source).map_or_else(Vec::new, |(tree, emitted)| fingerprint::units(&tree.root_node(), &source, path, &emitted.symbols, is_test_file(path)))
}
