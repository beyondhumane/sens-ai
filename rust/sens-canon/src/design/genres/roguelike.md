# Roguelikes and deckbuilders

This manual leans on what the developers of Dead Cells, Slay the Spire and Hades have said about their own games. None of them publishes a formula, so the numbers below are the few they gave, each with its context.

## Death and the next run

### Expectations

- Dying is part of the loop, not the end of the game: each run teaches something, and the next one starts at once.
- Time spent in a lost run never feels wasted.

### Systems

- Something survives each death: knowledge always; in a roguelite also unlocks and upgrades; in Hades also the story, which carries on from one death to the next.
- Reactive events: the game looks at the state of the run, such as whom the player meets or how much health is left, and picks one of several pre-written events, weighted so some come up more often.
- A fixed frame around the random parts: in Dead Cells the layout of the island and where the keys to new paths sit never change, whatever the seed.
- In Unity, keep what survives a death (unlocks, story flags, statistics) in one save object apart from the run's state, and build each run from a seed so it can be replayed and debugged.

### Numbers

- Supergiant set no number for this; Greg Kasavin states it as a goal, "to take the pain out of dying and having to restart". — [Game Developer](https://www.gamedeveloper.com/design/how-supergiant-weaves-narrative-rewards-into-i-hades-i-cycle-of-perpetual-death)

### Pitfalls

- A death that sends the player back with nothing to show: the moment of death turns into the moment they quit, which is what Kasavin set out to avoid.
- Upgrades that outweigh the decisions made in a run: the run stops mattering and only the grind does.

### References

- Hades: Greg Kasavin on structuring the story around dying and restarting, and on events chosen from the state of the run. — [Game Developer](https://www.gamedeveloper.com/design/how-supergiant-weaves-narrative-rewards-into-i-hades-i-cycle-of-perpetual-death)

## Procedural levels

Subgenres: roguelike, roguelite

### Expectations

- Each run is new, yet a level still feels designed: it has a logic, and an area keeps its identity from run to run.
- The player wins by reacting, not by memorising a layout.

### Systems

- Hybrid generation, in Dead Cells' six steps: a fixed world layout; handmade rooms, each built for one purpose; a concept graph per biome; the algorithm fills each node with random rooms of that biome until one fits; enemies placed by rules; loot last.
- Each handmade room declares its entrances, its exits and its purpose: a treasure room, a merchant and a combat room have different layouts.
- The biome's graph decides the level's shape: its length, how many special rooms it has, how labyrinthine it is and how many rooms separate the entrance from the nearest exit.
- Rooms belong to one biome, so each area keeps its identity: Dead Cells' sewers are tight on purpose, its ramparts straight.
- Pacing in dramatic peaks and calmer breaks, a philosophy Dead Cells kept from Left 4 Dead's AI Director.
- In Unity, rooms as prefabs whose entrances and purpose are data, the biome graph as a `ScriptableObject`, and a seeded `System.Random` per generator rather than the shared `UnityEngine.Random`, so a seed always rebuilds the same level.

### Numbers

- Dead Cells sets the number of enemies by the length of the combat rooms, with dangerous enemies counting for more; its own example, given as "random numbers", is 250 combat tiles at one monster per five tiles. — [The Level Design of Dead Cells](https://deepnight.net/tutorial/the-level-design-of-dead-cells-a-hybrid-approach/)

### Pitfalls

- Generating everything: Dead Cells' fully procedural prototype played well but its levels were "illogical, chaotic", with no feeling of consistency.
- Rooms shared across biomes: the areas blur into one another and the world loses its identity.
- Rules that let a seed produce an unfinishable level: every generated level needs a path to its exit.

### References

- Dead Cells: Sébastien Bénard's hybrid approach, step by step, crediting Spelunky for the idea. — [deepnight.net](https://deepnight.net/tutorial/the-level-design-of-dead-cells-a-hybrid-approach/)

## Balance through data

### Expectations

- Every option the game offers is worth considering, and no single build makes the game trivial.

### Systems

- Record every decision from the first prototype on: what was offered, what was picked over what, and how each run ended.
- Two measures per card or item: how often it is picked when offered, and how often it sits in a winning deck. Too rarely picked and it is not really in the game; too often in winning decks and it is too strong.
- For enemies, how much damage players take from each one, broken down by what they carry.
- Read the data for patterns, then decide by feel: the data says where to look, not what to change.
- Open the data with a specific question that playing cannot answer, and keep every view filterable.
- Read the few well-argued player reports too: the numbers do not say how something feels.
- In Unity, send one event per decision (offered, picked, run result) with the run's seed and the build version, so a balance change can be traced to the build that caused it.

### Numbers

- Slay the Spire's metrics started as three graphs and grew to at least 90. — [Game Developer](https://www.gamedeveloper.com/design/how-i-slay-the-spire-i-s-devs-use-data-to-balance-their-roguelike-deck-builder)
- In Early Access, one hour brought more data than the whole prototype phase. — [Game Developer](https://www.gamedeveloper.com/design/how-i-slay-the-spire-i-s-devs-use-data-to-balance-their-roguelike-deck-builder)

### Pitfalls

- Balancing by anecdote: players call a boss stronger or weaker depending on their own playstyle.
- Waiting to change things: Mega Crit preferred aggressive changes early in testing, when they are cheap.
- Treating the data as a formula: Anthony Giovannetti calls their use of it not "mathematical".

### References

- Slay the Spire: Anthony Giovannetti and Casey Yano on the metrics server behind every balance change. — [Game Developer](https://www.gamedeveloper.com/design/how-i-slay-the-spire-i-s-devs-use-data-to-balance-their-roguelike-deck-builder)
- Slay the Spire: Metrics Driven Design and Balance (GDC 2019, Anthony Giovannetti). — [GDC Vault](https://www.gdcvault.com/play/1025731/)

## Cards and synergies

Subgenres: deckbuilder

### Expectations

- Synergies the player discovers and builds a run around.
- A run is a chain of small choices: which card to take, or none.

### Systems

- Add cards in batches that form an archetype, then single cards that sculpt it.
- In a single-player game decks do not have to be equal, so a card can be tuned for fun rather than for parity.
- Infinite combos, where repeating cards kills an enemy in one turn, are the first thing to keep rare: they make the playing trivial.
- One small change can break an archetype: Slay the Spire's Dual Wield went from rarely picked to "totally broken" when it could copy any card in the hand, and had to be limited again.
- In Unity, cards as `ScriptableObject` data whose effects are composed from small reusable actions, so a balance change is a data change.

### Numbers

- Mega Crit gives no target pick rate in this interview; the measure is relative, and a card picked too rarely is "basically not a card in our game". — [Game Developer](https://www.gamedeveloper.com/design/how-i-slay-the-spire-i-s-devs-use-data-to-balance-their-roguelike-deck-builder)

### Pitfalls

- A card that copies or repeats other cards without limits: the shortest road to an infinite.
- Changing one card in isolation: every change ripples through the strategy that card belongs to.

### References

- Slay the Spire: the Dual Wield and Awakened One changes, as its developers tell them. — [Game Developer](https://www.gamedeveloper.com/design/how-i-slay-the-spire-i-s-devs-use-data-to-balance-their-roguelike-deck-builder)
