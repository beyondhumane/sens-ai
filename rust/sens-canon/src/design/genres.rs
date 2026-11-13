use super::{Section, markdown, sections};

struct Playbook {
    subgenres: &'static [&'static str],
    text: &'static str,
}

const PLAYBOOKS: [Playbook; 2] = [
    Playbook { subgenres: &["platformer", "metroidvania", "action-2d"], text: include_str!("genres/action-2d.md") },
    Playbook { subgenres: &["roguelike", "roguelite", "deckbuilder"], text: include_str!("genres/roguelike.md") },
];
const ONLY_FOR: &str = "Subgenres:";

fn only_for(text: &str) -> Option<Vec<&str>> {
    let first = text.lines().next()?.trim();
    first.strip_prefix(ONLY_FOR).map(|named| named.split(',').map(str::trim).filter(|name| !name.is_empty()).collect())
}

pub fn topics(subgenres: &[String]) -> Vec<Section> {
    let declared = |name: &str| subgenres.iter().any(|subgenre| subgenre == name);
    PLAYBOOKS
        .iter()
        .filter(|playbook| playbook.subgenres.iter().any(|name| declared(name)))
        .flat_map(|playbook| sections("", &markdown::blocks(playbook.text, 1)))
        .filter(|topic| only_for(&topic.text).is_none_or(|names| names.iter().any(|name| declared(name))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::SUBGENRES;
    use super::*;
    use markdown::Block;

    const PARTS: [&str; 5] = ["Expectations", "Systems", "Numbers", "Pitfalls", "References"];
    const TOPIC_LINES: u32 = 60;

    fn topic_blocks(blocks: &[Block]) -> Vec<(&Block, Vec<&Block>)> {
        let mut found: Vec<(&Block, Vec<&Block>)> = Vec::new();
        for block in blocks {
            match block.level {
                2 => found.push((block, Vec::new())),
                3 => found.last_mut().expect("a part before any topic").1.push(block),
                _ => {}
            }
        }
        found
    }

    #[test]
    fn every_playbook_keeps_its_form() {
        for playbook in &PLAYBOOKS {
            assert!(playbook.subgenres.iter().all(|name| SUBGENRES.contains(name)), "{:?}", playbook.subgenres);
            let blocks = markdown::blocks(playbook.text, 1);
            assert_eq!(blocks.iter().filter(|block| block.level == 1).count(), 1, "one title");
            assert!(blocks.iter().all(|block| block.level <= 3), "no heading deeper than ###");
            let topics = topic_blocks(&blocks);
            assert!(!topics.is_empty());
            let ends: Vec<u32> = topics.iter().skip(1).map(|(topic, _)| topic.line).chain([playbook.text.lines().count() as u32 + 1]).collect();
            for ((topic, parts), end) in topics.iter().zip(ends) {
                let titles: Vec<&str> = parts.iter().map(|part| part.title.as_str()).collect();
                assert_eq!(titles, PARTS, "{}", topic.title);
                assert!(end - topic.line <= TOPIC_LINES, "{} runs {} lines", topic.title, end - topic.line);
                if let Some(names) = only_for(topic.body.trim()) {
                    assert!(!names.is_empty() && names.iter().all(|name| playbook.subgenres.contains(name)), "{}: {names:?}", topic.title);
                }
                let numbers = parts[2].body.lines().filter(|line| line.trim_start().starts_with("- "));
                for bullet in numbers {
                    assert!(bullet.contains("](http"), "{}: a number without its source: {bullet}", topic.title);
                }
            }
        }
    }

    #[test]
    fn a_platformer_gets_the_shared_topics_but_not_the_metroidvania_ones() {
        let titles = |subgenres: &[&str]| topics(&subgenres.iter().map(|name| name.to_string()).collect::<Vec<_>>()).into_iter().map(|topic| topic.title).collect::<Vec<_>>();
        let platformer = titles(&["platformer"]);
        assert!(platformer.contains(&"Action 2D and platformers › Jump".to_string()), "{platformer:?}");
        assert!(!platformer.iter().any(|title| title.ends_with("› Metroidvania world") || title.ends_with("› Combat feedback")), "{platformer:?}");
        assert!(titles(&["metroidvania"]).contains(&"Action 2D and platformers › Metroidvania world".to_string()));
        let roguelike = titles(&["roguelike"]);
        assert!(roguelike.contains(&"Roguelikes and deckbuilders › Procedural levels".to_string()) && !roguelike.iter().any(|title| title.starts_with("Action 2D") || title.ends_with("› Cards and synergies")), "{roguelike:?}");
        let deckbuilder = titles(&["deckbuilder"]);
        assert!(deckbuilder.contains(&"Roguelikes and deckbuilders › Cards and synergies".to_string()) && !deckbuilder.iter().any(|title| title.ends_with("› Procedural levels")), "{deckbuilder:?}");
        assert!(topics(&["platformer".to_string()]).iter().all(|topic| topic.file.is_empty()));
    }
}
