use std::collections::HashMap;

use super::{Design, Section, first_paragraph, genres};
use crate::rank::{self, Corpus, words};

pub const CANON: &str = include_str!("canon.md");
const PILLARS: &str = "Design/pillars.md";
const LOOP: &str = "Design/loop.md";
const SECTIONS: usize = 3;
const LINES: usize = 80;
const TITLE_WEIGHT: usize = 3;
const FLATTEN: f32 = 0.75;

pub struct Guide {
    design: Design,
    searched: Vec<Section>,
    corpus: Corpus,
}

pub struct Told<'g> {
    pub text: String,
    pub sections: Vec<&'g Section>,
}

fn terms(section: &Section) -> HashMap<String, usize> {
    let mut terms: HashMap<String, usize> = HashMap::new();
    for word in words(&section.title) {
        *terms.entry(word).or_default() += TITLE_WEIGHT;
    }
    for word in words(&section.text) {
        *terms.entry(word).or_default() += 1;
    }
    terms
}

fn is_pillars(section: &Section) -> bool {
    section.file.eq_ignore_ascii_case(PILLARS)
}

impl Guide {
    pub fn of(design: Design) -> Guide {
        let searched: Vec<Section> = design.sections.iter().filter(|section| !is_pillars(section)).cloned().chain(genres::topics(&design.subgenres)).collect();
        let corpus = Corpus::of(searched.iter().map(terms).collect());
        Guide { design, searched, corpus }
    }

    pub fn briefing(&self) -> String {
        let mut said = vec![CANON.trim_end().to_string()];
        let pillars: Vec<String> = self.design.sections.iter().filter(|section| is_pillars(section)).map(|section| format!("**{}**\n{}", section.title, section.text)).collect();
        if !pillars.is_empty() {
            said.push(format!("The pillars of this game, from {PILLARS}:\n\n{}", pillars.join("\n\n")));
        }
        if !self.design.subgenres.is_empty() {
            said.push(format!("Subgenres: {}.", self.design.subgenres.join(", ")));
        }
        if !self.design.warnings.is_empty() {
            let warnings: Vec<String> = self.design.warnings.iter().map(|warning| format!("- {warning}")).collect();
            said.push(format!("Sens found these problems in `Design/`. Tell the person about them once:\n{}", warnings.join("\n")));
        }
        said.join("\n\n")
    }

    pub fn overview(&self) -> String {
        let design = &self.design;
        let mut said = Vec::new();
        let pillars: Vec<String> = design
            .pillars
            .iter()
            .map(|pillar| format!("- {} [{}] — {}{}", pillar.title, pillar.id, pillar.statement, if pillar.rules_out { "" } else { " (rules nothing out)" }))
            .collect();
        said.push(if pillars.is_empty() { format!("Pillars: none yet ({PILLARS}).") } else { format!("Pillars ({PILLARS}):\n{}", pillars.join("\n")) });
        if !design.subgenres.is_empty() {
            said.push(format!("Subgenres: {}.", design.subgenres.join(", ")));
        }
        let looped: Vec<String> = design.sections.iter().filter(|section| section.file.eq_ignore_ascii_case(LOOP)).map(|section| format!("- {}: {}", section.title, first_paragraph(&section.text))).collect();
        said.push(if looped.is_empty() { format!("Core loop: not written yet ({LOOP}).") } else { format!("Core loop ({LOOP}):\n{}", looped.join("\n")) });
        let mechanics: Vec<String> = design
            .mechanics
            .iter()
            .map(|mechanic| {
                let status = mechanic.status.map_or("no status", |status| status.name());
                let served = if mechanic.pillars.is_empty() { "no pillar".to_string() } else { mechanic.pillars.join(", ") };
                format!("- {} — {status} — serves {served} — {}", mechanic.title, mechanic.file)
            })
            .collect();
        said.push(if mechanics.is_empty() { "Mechanics: none yet (Design/mechanics/).".to_string() } else { format!("Mechanics:\n{}", mechanics.join("\n")) });
        if !design.warnings.is_empty() {
            said.push(format!("Problems Sens found in Design/:\n{}", design.warnings.iter().map(|warning| format!("- {warning}")).collect::<Vec<_>>().join("\n")));
        }
        said.join("\n\n")
    }

    pub fn relevant(&self, prompt: &str) -> Option<Told<'_>> {
        let groups = self.corpus.wanted(prompt);
        if groups.is_empty() {
            return None;
        }
        let mut scored: Vec<(usize, f32)> = (0..self.searched.len()).map(|at| (at, self.corpus.score(at, &groups, FLATTEN))).collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let mut left = LINES;
        let mut told = Told { text: "From this game's design (`Design/`), related to the request:".to_string(), sections: Vec::new() };
        for (at, _) in rank::kept(scored, |found| found.1, SECTIONS) {
            if left < 2 {
                break;
            }
            let section = &self.searched[at];
            let lines: Vec<&str> = section.text.lines().collect();
            let fits = lines.len().min(left - 1);
            let manual = section.file.is_empty();
            let from = if manual { "Sens's genre manual".to_string() } else { format!("{}:{}", section.file, section.line) };
            let cut = match (fits < lines.len(), manual) {
                (false, _) => "",
                (true, true) => " (cut to fit; ask `design_search` for the rest)",
                (true, false) => " (cut to fit; read the file for the rest)",
            };
            told.text.push_str(&format!("\n\n### {} — {from}{cut}\n{}", section.title, lines[..fits].join("\n")));
            left -= 1 + fits;
            told.sections.push(section);
        }
        (!told.sections.is_empty()).then_some(told)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::Design;
    use sens_index::fixture::written;

    const PILLARS_MD: (&str, &str) = ("Design/pillars.md", "## Movimiento preciso\n\nEl jugador siempre sabe por qué falló un salto.\n\n### Descarta\n\nAleatoriedad en el control.\n");
    const DASH: (&str, &str) = ("Design/mechanics/dash.md", "---\npillars: [movimiento-preciso]\nstatus: prototype\n---\n# Dash\n\nUn impulso corto en el aire que se recarga al tocar el suelo.\n");
    const SHOP: (&str, &str) = ("Design/economy.md", "# Tienda\n\nLas monedas se gastan entre niveles.\n");

    fn guide(name: &str, files: &[(&str, &str)]) -> Guide {
        Guide::of(Design::read(&written("sens-canon-guide", name, files)).unwrap())
    }

    #[test]
    fn the_canon_text_says_its_three_things_in_few_lines() {
        assert!(CANON.lines().count() <= 12, "{} lines", CANON.lines().count());
        assert!(CANON.contains("`Design/` is the source of truth"));
        assert!(CANON.contains("update its file in `Design/mechanics/`"));
        assert!(CANON.contains("goes against a pillar, say so before writing"));
    }

    #[test]
    fn the_briefing_carries_the_canon_text_the_pillars_with_what_they_rule_out_and_the_problems() {
        let briefing = guide("briefing", &[("Design/README.md", "---\nsubgenres: [platformer, soulslike]\n---\n# Pájaro\n"), PILLARS_MD, DASH]).briefing();
        assert!(briefing.starts_with("## This game's design"));
        assert!(briefing.contains("**Movimiento preciso**\nEl jugador siempre sabe por qué falló un salto.") && briefing.contains("Aleatoriedad en el control."), "{briefing}");
        assert!(briefing.contains("Subgenres: platformer."));
        assert!(briefing.contains("Tell the person about them once:\n- Design/README.md: `soulslike` is not a subgenre"), "{briefing}");
    }

    #[test]
    fn a_request_brings_the_sections_it_is_about_and_never_the_pillars_again() {
        let guide = guide("relevant", &[PILLARS_MD, DASH, SHOP]);
        let told = guide.relevant("haz que el dash se recargue en el aire").unwrap();
        assert_eq!(told.sections.iter().map(|section| section.title.as_str()).collect::<Vec<_>>(), ["Dash"]);
        assert!(told.text.contains("### Dash — Design/mechanics/dash.md:5\nUn impulso corto"), "{}", told.text);
        assert!(guide.relevant("añade una tienda para gastar monedas").is_some_and(|told| told.sections[0].title == "Tienda"));
        assert!(guide.relevant("el jugador siempre sabe por qué falló").is_none_or(|told| told.sections.iter().all(|section| !section.file.ends_with("pillars.md"))));
        assert!(guide.relevant("hola, ¿qué tal?").is_none());
    }

    #[test]
    fn a_platformer_request_brings_the_manual_topic_and_a_game_without_subgenres_does_not() {
        let platformer = guide("manual", &[("Design/README.md", "---\nsubgenres: [platformer]\n---\n# Pájaro\n")]);
        let told = platformer.relevant("añade doble salto").unwrap();
        assert_eq!(told.sections[0].title, "Action 2D and platformers › Jump");
        assert!(told.sections[0].file.is_empty());
        assert!(told.text.contains("### Action 2D and platformers › Jump — Sens's genre manual\n"), "{}", told.text);
        assert!(guide("no-manual", &[("Design/README.md", "# Pájaro\n")]).relevant("añade doble salto").is_none());
    }

    #[test]
    fn never_more_than_three_sections_nor_more_than_eighty_lines() {
        let long = |name: &str| format!("# Dash {name}\n\n{}", "El dash cruza el hueco.\n".repeat(50));
        let files: Vec<(String, String)> = ["a", "b", "c", "d", "e"].iter().map(|name| (format!("Design/mechanics/{name}.md"), long(name))).collect();
        let borrowed: Vec<(&str, &str)> = files.iter().map(|(path, text)| (path.as_str(), text.as_str())).collect();
        let guide = guide("budget", &borrowed);
        let told = guide.relevant("el dash").unwrap();
        assert!(told.sections.len() <= SECTIONS);
        let section_lines = told.text.lines().count() - 1 - told.sections.len();
        assert!(section_lines <= LINES, "{section_lines} lines");
        assert!(told.text.contains("(cut to fit; read the file for the rest)"));
    }
}
