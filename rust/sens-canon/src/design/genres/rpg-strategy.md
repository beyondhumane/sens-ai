# RPGs and strategy

This manual leans on what the designers of Into the Breach, Civilization and Pillars of Eternity have said about their own games, in their own writing or as reported from their talks.

## Readable threats

Subgenres: tactics, strategy

### Expectations

- The player knows what the enemy will do next turn and has a chance to stop it.
- Losing is the player's mistake, not a surprise the game kept hidden.

### Systems

- Show every enemy's next attack before the player moves, so each turn is a problem to solve rather than a guess.
- Make the threat land on something the player has to protect. In Into the Breach the enemies attack cities: when only the player's mechs were targets, the danger was trivial because the mechs could simply walk away.
- Tie the stakes to a mechanic, not to atmosphere: losing a city costs a point of the power grid, which is the player's life.
- Keep every number small: when everything scales up together, nothing changes except how hard the board is to read.
- Let the units be expendable when the objective matters more: a lost mech costs its pilot and comes back with a pilot that cannot earn experience.
- In Unity, compute every enemy's intended action at the start of the player's turn and keep it as data the board draws from, so what is shown is exactly what will happen.

### Numbers

- Into the Breach: the power grid holds just seven points when full; longer or upgradable bars made players stop caring about damage. — [Game Developer](https://www.gamedeveloper.com/design/reimagining-failure-in-strategy-game-design-in-i-into-the-breach-i-)

### Pitfalls

- Making it hard by filling the board with enemies and danger zones: it becomes "needlessly complex and hard to parse", in Justin Ma's words.
- Stakes that live only in the presentation: decoration cannot make the player care the way a mechanic does.

### References

- Into the Breach: Justin Ma and Matthew Davis on telegraphed attacks, cities as targets and expendable mechs. — [Game Developer](https://www.gamedeveloper.com/design/reimagining-failure-in-strategy-game-design-in-i-into-the-breach-i-)

## Odds the player trusts

### Expectations

- A chance shown on screen feels like what it says.
- A bad roll is part of the game, not a reason to reload.

### Systems

- Players read odds through psychology, not arithmetic: Sid Meier's testers accepted losing at two to one but were upset losing at 20 to 10, and never questioned winning at one to three.
- So a combat system cannot be only a maths system; design what a result feels like, not just its probability.
- Big swings of fortune push players to reload: Meier's planned rise and fall for the first Civilization, a near disaster followed by recovery, only made players load an old save.
- Saving the random seed with the game makes a reloaded combat play out the same way; Civilization III introduced it, and after players objected it became an option at game start.
- In Unity, keep one seeded `System.Random` per game, stored in the save, and give the player the choice of a fixed or a fresh seed on reload.

### Numbers

- Civilization Revolution's testers: a loss at 2 to 1 was accepted, a loss at 20 to 10 was not, though the odds are the same. — [Game Informer](https://www.gameinformer.com/b/news/archive/2010/03/12/sid-meier-gamers-arent-logical.aspx)

### Pitfalls

- Odds shown as exact numbers with no thought for how they feel: every unlikely loss feels unfair.
- Forcing the seed on everyone: some Civilization III players felt the game was cheating them by always repeating the same result.

### References

- The Psychology of Game Design (Sid Meier, GDC 2010), as reported by Game Informer. — [Game Informer](https://www.gameinformer.com/b/news/archive/2010/03/12/sid-meier-gamers-arent-logical.aspx)
- Water Finds a Crack (Soren Johnson): the saved random seed and the players' reaction. — [Designer Notes](https://www.designer-notes.com/game-developer-column-17-water-finds-a-crack/)

## Dominant strategies

### Expectations

- Every decision is right in some circumstances and wrong in others, with a wide grey area between.
- The optimal way to play is also a fun way to play.

### Systems

- Given the chance, players will optimise the fun out of a game; any hole in the design will be found and abused, and the Civilization team called this "water finds a crack".
- A single dominant strategy removes choice, because every other option becomes provably worse.
- Counter a dominant strategy with a rule that makes it costly, as Civilization IV did with a city maintenance cost that grows with the number of cities.
- Remove the incentive for tedious optimisation: Civilization IV carries leftover food, production and research over to the next item, so nothing is lost by not micromanaging.
- Not every exploit is bad: one that is a fun shortcut and does not drown other ways to play can stay, or become an option the player turns on.
- In Unity, log which strategies win and how long they take, so a dominant one shows up in the data before it shows up in the reviews.

### Numbers

- Civilization: 50 cities of size 2 beat 5 cities of size 20, because each city works its central tile for free, 1.5 tiles per citizen against 1.05. — [Designer Notes](https://www.designer-notes.com/game-developer-column-17-water-finds-a-crack/)
- Civilization: at 21 research a turn, a 100-point technology throws 5 points away, which pushed players to adjust taxes and rearrange their cities every turn. — [Designer Notes](https://www.designer-notes.com/game-developer-column-17-water-finds-a-crack/)

### Pitfalls

- Rewards for repeating a safe action: players trade their time for safety until the game becomes a grind.
- Trying to remove every exploit: some are the most fun part, like Super Mario Bros.' infinite lives trick, which Nintendo left in on purpose.

### References

- Water Finds a Crack (Soren Johnson, Game Developer, March 2011): tank-mages, infinite city sleaze, overflow and saved seeds. — [Designer Notes](https://www.designer-notes.com/game-developer-column-17-water-finds-a-crack/)

## Attributes and builds

Subgenres: rpg, action-rpg

### Expectations

- No character the player builds is a bad build: every one can finish the game.
- Every attribute matters to every class, so none is the stat everyone sacrifices.

### Systems

- Separate viable, a character that survives the game, from optimal, how far a min-maxer can push it; design to shrink the gap between the two.
- Make each attribute affect every class's actions: in Pillars of Eternity, Might raises all damage, from weapons and spells alike.
- Choose between attributes by what a build needs, such as accuracy against damage, not by which class the character belongs to.
- Keep attributes less extreme than in tabletop rules, so the ends of the range stay balanced.
- Companions are middle-of-the-road, so they pair well with a min-maxed player character of the same class.
- Point the player at an attribute habit tells them to drop: Pillars put a star next to Might for wizards.
- In Unity, attributes as data that feed a single damage and accuracy formula shared by every class, so no class can quietly ignore one.

### Numbers

- Pillars of Eternity kept attributes less impactful than D&D: 1d10 + 24 % against 1d10 + 6, to keep the extremes balanced. — [GameBanshee, reporting Sawyer's GDC talk](https://www.gamebanshee.com/news/116980-pillars-of-eternity-attribute-design-gdc-presentation-reports.html)

### Pitfalls

- An attribute only some classes need: everyone else dumps it, and the content built around it loses its point.
- Treating a dumped stat as the player's fault: Josh Sawyer calls fixing dump stats a reality check on the whole system.

### References

- Gods and Dumps: Attribute Tuning in Pillars of Eternity (GDC 2016, Josh Sawyer). — [GDC Vault](https://gdcvault.com/play/1023481/Gods-and-Dumps-Attribute-Tuning)
- The same talk, as reported from the room by PCGamesN and Soren Johnson. — [GameBanshee](https://www.gamebanshee.com/news/116980-pillars-of-eternity-attribute-design-gdc-presentation-reports.html)
