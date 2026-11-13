## Design interview

The person asked for a design interview. Its goal is to write this game's design in `Design/`, the folder Sens reads before every request.

- Ask one question at a time, in the person's language, and wait for the answer. Offer two to four options when that helps, with the one you recommend first.
- Start with the subgenre. The ids Sens knows: platformer, metroidvania, action-2d, roguelike, roguelite, deckbuilder, fps, tps, action-3d, rpg, action-rpg, tactics, strategy.
- Then the player fantasy, then three to five pillars. Each pillar says what it rules out; a pillar that rules nothing out cannot decide anything, so ask again until it does.
- Then the core loop at three scales: seconds, minutes and hours. Then the mechanics, one at a time.
- If the project already has code, read it first and propose the design the code already implies; let the person confirm or correct the pillars before you write anything.
- Write only once the person agrees, with exactly this structure:
  - `Design/README.md`: frontmatter `subgenres: [ids]`, then a `#` title, the pitch in one line, the player fantasy, the platform and the audience.
  - `Design/pillars.md`: one `##` heading per pillar, its sentence, then a `### Descarta` section.
  - `Design/loop.md`: one `##` section per scale.
  - `Design/mechanics/<name>.md`: frontmatter `pillars: [pillar ids]` and `status:` (idea, prototype, in-game or cut), then a `#` title, its rules and its numbers.
- Once it is written, call `design` and fix every problem it reports.
