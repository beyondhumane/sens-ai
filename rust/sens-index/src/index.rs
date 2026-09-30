use std::collections::HashMap;
use std::path::PathBuf;

use serde::Serialize;

pub const NOBODY: u32 = u32::MAX;
pub const EVERYTHING: &str = "*";

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SymbolInfo {
    pub id: String,
    pub kind: String,
    pub entry: bool,
    pub name: String,
    pub file: String,
    pub line: u32,
    pub end_line: u32,
    pub start_byte: usize,
    pub end_byte: usize,
    pub signature: String,
    pub exported: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Reference {
    pub file: String,
    pub line: u32,
    pub from: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub language: &'static str,
    pub mtime_ms: f64,
    pub exports: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ImportEdge {
    pub from: String,
    pub to: String,
    pub names: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Index {
    pub root: PathBuf,
    pub files: Vec<FileInfo>,
    pub symbols: Vec<SymbolInfo>,
    pub imports: Vec<ImportEdge>,
    pub entry_points: Vec<String>,
    sites: Vec<Vec<(u32, u32, u32)>>,
    by_id: HashMap<String, usize>,
}

impl Index {
    pub fn assemble(root: PathBuf, files: Vec<FileInfo>, symbols: Vec<SymbolInfo>, imports: Vec<ImportEdge>, mut references: HashMap<String, Vec<Reference>>) -> Index {
        let file_slot: HashMap<&str, u32> = files.iter().enumerate().map(|(at, file)| (file.path.as_str(), at as u32)).collect();
        let by_id: HashMap<String, usize> = symbols.iter().enumerate().map(|(at, symbol)| (symbol.id.clone(), at)).collect();
        let mut sites: Vec<Vec<(u32, u32, u32)>> = symbols
            .iter()
            .map(|symbol| {
                references
                    .remove(&symbol.id)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|reference| {
                        let file = *file_slot.get(reference.file.as_str())?;
                        let from = reference.from.and_then(|from| by_id.get(&from).map(|&at| at as u32)).unwrap_or(NOBODY);
                        Some((file, reference.line, from))
                    })
                    .collect()
            })
            .collect();
        for edge in imports.iter().filter(|edge| edge.names.iter().any(|name| name == EVERYTHING)) {
            let Some(&from) = file_slot.get(edge.from.as_str()) else {
                continue;
            };
            for (at, symbol) in symbols.iter().enumerate() {
                if symbol.exported && symbol.file == edge.to {
                    sites[at].push((from, 1, NOBODY));
                }
            }
        }
        Index { root, files, symbols, imports, entry_points: Vec::new(), sites, by_id }
    }

    pub fn raw_references(&self, symbol: usize) -> &[(u32, u32, u32)] {
        self.sites.get(symbol).map_or(&[], Vec::as_slice)
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.by_id.get(id).copied()
    }

    pub fn references_at(&self, symbol: usize) -> Vec<Reference> {
        self.raw_references(symbol)
            .iter()
            .map(|&(file, line, from)| Reference {
                file: self.files[file as usize].path.clone(),
                line,
                from: (from != NOBODY).then(|| self.symbols[from as usize].id.clone()),
            })
            .collect()
    }

    pub fn references_for(&self, id: &str) -> Vec<Reference> {
        self.index_of(id).map(|symbol| self.references_at(symbol)).unwrap_or_default()
    }
}
