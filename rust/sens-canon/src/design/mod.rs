mod frontmatter;
mod guide;
mod markdown;
mod undescribed;

pub use guide::{Guide, Told};
pub use undescribed::undescribed;

pub const INTERVIEW: &str = include_str!("interview.md");

use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use crate::slug::slug;
use markdown::Block;

const FOLDER: &str = "Design";
const SUBGENRES: [&str; 13] = ["platformer", "metroidvania", "action-2d", "roguelike", "roguelite", "deckbuilder", "fps", "tps", "action-3d", "rpg", "action-rpg", "tactics", "strategy"];
const RULES_OUT: [&str; 6] = ["descarta", "rules out", "exclut", "schließt aus", "除外", "排除"];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idea,
    Prototype,
    InGame,
    Cut,
}

impl Status {
    const ALL: [Status; 4] = [Status::Idea, Status::Prototype, Status::InGame, Status::Cut];

    pub fn name(self) -> &'static str {
        match self {
            Status::Idea => "idea",
            Status::Prototype => "prototype",
            Status::InGame => "in-game",
            Status::Cut => "cut",
        }
    }

    fn named(name: &str) -> Option<Status> {
        Status::ALL.into_iter().find(|status| status.name() == name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pillar {
    pub id: String,
    pub title: String,
    pub statement: String,
    pub line: u32,
    pub rules_out: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mechanic {
    pub file: String,
    pub title: String,
    pub pillars: Vec<String>,
    pub status: Option<Status>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    pub file: String,
    pub line: u32,
    pub title: String,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Design {
    pub subgenres: Vec<String>,
    pub pillars: Vec<Pillar>,
    pub mechanics: Vec<Mechanic>,
    pub sections: Vec<Section>,
    pub warnings: Vec<String>,
}

pub fn pillar_id(title: &str) -> String {
    let id = slug(title);
    if id.is_empty() { title.trim().to_lowercase() } else { id }
}

fn markdown_files(folder: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else { return };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        if path.is_dir() {
            markdown_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("md")) {
            out.push(path);
        }
    }
}

fn first_paragraph(body: &str) -> String {
    body.split("\n\n").map(str::trim).find(|paragraph| !paragraph.is_empty()).unwrap_or_default().split_whitespace().collect::<Vec<_>>().join(" ")
}

fn stem(file: &str) -> &str {
    let name = file.rsplit('/').next().unwrap_or(file);
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

fn sections(file: &str, blocks: &[Block]) -> Vec<Section> {
    let mut found: Vec<Section> = Vec::new();
    let mut top: Option<&str> = None;
    for block in blocks {
        if block.level >= 3
            && let Some(open) = found.last_mut()
        {
            open.text.push_str(&format!("\n\n### {}\n{}", block.title, block.body.trim()));
            continue;
        }
        let title = match block.level {
            0 => stem(file).to_string(),
            1 => {
                top = Some(block.title.as_str());
                block.title.clone()
            }
            _ => top.map_or_else(|| block.title.clone(), |top| format!("{top} › {}", block.title)),
        };
        found.push(Section { file: file.to_string(), line: block.line, title, text: block.body.trim().to_string() });
    }
    found.into_iter().filter(|section| !section.text.trim().is_empty()).collect()
}

impl Design {
    pub fn read(root: &Path) -> Option<Design> {
        let folder = root.join(FOLDER);
        if !folder.is_dir() {
            return None;
        }
        let mut files = Vec::new();
        markdown_files(&folder, &mut files);
        files.sort();
        let mut design = Design::default();
        for path in files {
            let file = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace(MAIN_SEPARATOR, "/");
            let Ok(text) = std::fs::read_to_string(&path) else {
                design.warnings.push(format!("{file} is not readable as UTF-8 text, so Sens skipped it."));
                continue;
            };
            let front = match frontmatter::read(&text) {
                Ok(front) => front,
                Err(why) => {
                    design.warnings.push(format!("{file}: {why}, so Sens skipped it."));
                    continue;
                }
            };
            for line in &front.odd {
                design.warnings.push(format!("{file}:{line}: a frontmatter line must be `key: value` or `key: [a, b]`; Sens ignored this one."));
            }
            let blocks = markdown::blocks(front.body, front.body_line);
            let inside = file[FOLDER.len() + 1..].to_lowercase();
            if inside == "readme.md" {
                design.declare(front.fields.get("subgenres").map_or(&[][..], Vec::as_slice));
            } else if inside == "pillars.md" {
                design.pillars_from(&file, &blocks);
            } else if inside.starts_with("mechanics/") {
                design.mechanic(&file, &front.fields, &blocks);
            }
            design.sections.extend(sections(&file, &blocks));
        }
        design.check_pillars_served();
        Some(design)
    }

    fn declare(&mut self, named: &[String]) {
        let mut unknown: Vec<String> = Vec::new();
        for name in named.iter().map(|name| name.to_lowercase()) {
            let known = SUBGENRES.contains(&name.as_str());
            let seen = if known { &mut self.subgenres } else { &mut unknown };
            if !seen.contains(&name) {
                seen.push(name);
            }
        }
        for name in unknown {
            self.warnings.push(format!("{FOLDER}/README.md: `{name}` is not a subgenre Sens knows, so it was ignored. Known subgenres: {}.", SUBGENRES.join(", ")));
        }
    }

    fn pillars_from(&mut self, file: &str, blocks: &[Block]) {
        for block in blocks {
            match block.level {
                2 => self.pillars.push(Pillar { id: pillar_id(&block.title), title: block.title.clone(), statement: first_paragraph(&block.body), line: block.line, rules_out: false }),
                0 | 1 => {}
                _ => {
                    if let Some(pillar) = self.pillars.last_mut().filter(|_| RULES_OUT.contains(&block.title.to_lowercase().as_str())) {
                        pillar.rules_out = true;
                    }
                }
            }
        }
        for pillar in self.pillars.iter().filter(|pillar| !pillar.rules_out) {
            self.warnings.push(format!(
                "{file}:{}: the pillar `{}` does not say what it rules out. A pillar that rules nothing out cannot decide anything: add a `### Descarta` section under it.",
                pillar.line, pillar.title
            ));
        }
    }

    fn mechanic(&mut self, file: &str, fields: &std::collections::BTreeMap<String, Vec<String>>, blocks: &[Block]) {
        let title = blocks.iter().find(|block| block.level == 1).map_or_else(|| stem(file).to_string(), |block| block.title.clone());
        let named = fields.get("status").and_then(|values| values.first()).map(|status| status.to_lowercase());
        let status = named.as_deref().and_then(Status::named);
        match (&named, status) {
            (None, _) => self.warnings.push(format!("{file} has no `status:`. Say whether the mechanic is an idea, a prototype, in-game or cut.")),
            (Some(named), None) => self.warnings.push(format!("{file}: the status `{named}` is not one of idea, prototype, in-game or cut.")),
            _ => {}
        }
        let pillars = fields.get("pillars").map_or_else(Vec::new, |pillars| pillars.iter().map(|pillar| pillar_id(pillar)).collect());
        self.mechanics.push(Mechanic { file: file.to_string(), title, pillars, status });
    }

    fn check_pillars_served(&mut self) {
        for mechanic in &self.mechanics {
            for served in mechanic.pillars.iter().filter(|served| !self.pillars.iter().any(|pillar| &pillar.id == *served)) {
                self.warnings.push(format!("{}: it serves the pillar `{served}`, which {FOLDER}/pillars.md does not have.", mechanic.file));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_index::fixture::written;

    const README: (&str, &str) = ("Design/README.md", "---\nsubgenres: [platformer, Metroidvania]\n---\n# Pájaro\n\nUn pájaro que no sabe volar escala una torre.\n");
    const PILLARS: (&str, &str) = (
        "Design/pillars.md",
        "# Pilares\n\n## Movimiento preciso\n\nEl jugador siempre sabe por qué falló un salto.\n\n### Descarta\n\nAleatoriedad en el control.\n\n## Exploración libre\n\nCada sala tiene más de una salida.\n\n### Rules out\n\nPasillos.\n\n## Riesgo\n\nArriesgar siempre paga.\n",
    );
    const LOOP: (&str, &str) = ("Design/loop.md", "# Bucle\n\n## Segundos\n\nSaltar, agarrarse, impulsarse.\n");
    const DASH: (&str, &str) = (
        "Design/mechanics/dash.md",
        "---\npillars: [movimiento-preciso, Exploración libre]\nstatus: prototype\n---\n# Dash\n\nUn impulso corto en el aire.\n\n## Números\n\nDura 0,15 s.\n\n### Coste\n\nUno por salto.\n",
    );

    fn design(name: &str, files: &[(&str, &str)]) -> Design {
        Design::read(&written("sens-canon-design", name, files)).expect("Design/ exists")
    }

    #[test]
    fn a_whole_design_is_read_into_its_parts() {
        let read = design("whole", &[README, PILLARS, LOOP, DASH]);
        assert_eq!(read.subgenres, ["platformer", "metroidvania"]);
        let pillars: Vec<(&str, bool)> = read.pillars.iter().map(|pillar| (pillar.id.as_str(), pillar.rules_out)).collect();
        assert_eq!(pillars, [("movimiento-preciso", true), ("exploracion-libre", true), ("riesgo", false)]);
        assert_eq!(read.pillars[0].statement, "El jugador siempre sabe por qué falló un salto.");
        assert_eq!(read.mechanics, [Mechanic { file: "Design/mechanics/dash.md".into(), title: "Dash".into(), pillars: vec!["movimiento-preciso".into(), "exploracion-libre".into()], status: Some(Status::Prototype) }]);
        assert_eq!(read.warnings.len(), 1, "{:?}", read.warnings);
        assert!(read.warnings[0].starts_with("Design/pillars.md:19: the pillar `Riesgo` does not say what it rules out"), "{}", read.warnings[0]);
        let dash: Vec<&str> = read.sections.iter().filter(|section| section.file.ends_with("dash.md")).map(|section| section.title.as_str()).collect();
        assert_eq!(dash, ["Dash", "Dash › Números"]);
        assert!(read.sections.iter().any(|section| section.title == "Dash › Números" && section.text.contains("### Coste\nUno por salto.")));
    }

    #[test]
    fn without_a_design_folder_there_is_no_design() {
        assert_eq!(Design::read(&written("sens-canon-design", "none", &[("Assets/Player.cs", "class Player {}")])), None);
    }

    #[test]
    fn an_unknown_subgenre_is_ignored_and_said_once() {
        let read = design("subgenre", &[("Design/README.md", "---\nsubgenres: [platformer, soulslike, soulslike]\n---\n# Juego\n")]);
        assert_eq!(read.subgenres, ["platformer"]);
        let said: Vec<&String> = read.warnings.iter().filter(|warning| warning.contains("`soulslike` is not a subgenre")).collect();
        assert_eq!(said.len(), 1);
    }

    #[test]
    fn a_broken_file_is_skipped_and_said_while_the_rest_is_read() {
        let root = written("sens-canon-design", "broken", &[README, PILLARS, DASH, ("Design/mechanics/open.md", "---\nstatus: idea\n# Never closed\n")]);
        std::fs::write(root.join("Design/mechanics/binary.md"), [0xff, 0xfe, 0x00, 0x23]).unwrap();
        let read = Design::read(&root).unwrap();
        assert_eq!(read.mechanics.iter().map(|mechanic| mechanic.title.as_str()).collect::<Vec<_>>(), ["Dash"]);
        assert!(read.warnings.iter().any(|warning| warning == "Design/mechanics/binary.md is not readable as UTF-8 text, so Sens skipped it."));
        assert!(read.warnings.iter().any(|warning| warning.starts_with("Design/mechanics/open.md: its frontmatter opens with `---` and never closes")));
        assert_eq!(read.pillars.len(), 3);
    }

    #[test]
    fn a_mechanic_says_what_is_wrong_with_its_frontmatter() {
        let read = design(
            "mechanics",
            &[
                PILLARS,
                ("Design/mechanics/wall.md", "---\npillars: [ritmo]\nstatus: wip\n---\n# Wall jump\n"),
                ("Design/mechanics/hook.md", "---\npillars:\n  - riesgo\n---\n# Gancho\n"),
            ],
        );
        let said = read.warnings.join("\n");
        assert!(said.contains("Design/mechanics/wall.md: the status `wip` is not one of"), "{said}");
        assert!(said.contains("Design/mechanics/wall.md: it serves the pillar `ritmo`, which Design/pillars.md does not have."), "{said}");
        assert!(said.contains("Design/mechanics/hook.md has no `status:`"), "{said}");
        assert!(said.contains("Design/mechanics/hook.md:3: a frontmatter line must be"), "{said}");
    }

    #[test]
    fn the_interview_is_short_and_asks_for_exactly_what_the_reader_reads() {
        assert!(INTERVIEW.lines().count() <= 25, "{} lines", INTERVIEW.lines().count());
        for subgenre in SUBGENRES {
            assert!(INTERVIEW.contains(subgenre), "{subgenre}");
        }
        for status in Status::ALL {
            assert!(INTERVIEW.contains(status.name()), "{}", status.name());
        }
        for part in ["`Design/README.md`", "`Design/pillars.md`", "`Design/loop.md`", "`Design/mechanics/", "subgenres:", "pillars:", "status:", "### Descarta"] {
            assert!(INTERVIEW.contains(part), "{part}");
        }
        assert!(RULES_OUT.contains(&"descarta"));
    }

    #[test]
    fn pillar_ids_ignore_case_and_accents_and_keep_scripts_without_them() {
        assert_eq!(pillar_id("Exploración Libre"), "exploracion-libre");
        assert_eq!(pillar_id("正確な移動"), "正確な移動");
    }
}
