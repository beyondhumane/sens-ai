# Action 3D and shooters

This manual leans on what id Software, Santa Monica Studio, The Astronauts, Valve and Riot have published about their own games. Where they describe a system without giving its values, the manual says so instead of inventing them.

## Combat that pushes forward

### Expectations

- The player wins by moving into the fight, not by waiting behind cover.
- Resources come from the fight itself, so running low is a reason to attack rather than to retreat.

### Systems

- A filter for every mechanic: anything that makes the player stop and pause is removed. id calls its loop "push-forward combat".
- No regenerating health: health and ammo come from enemies, so the player has to move towards them.
- Finishing moves that pay more health and ammo than a kill by shooting, and that change with the player's position.
- Strict time limits on anything that takes control away from the player, so the action is never interrupted for long.
- In Unity, finishing moves as short animation states with a hard maximum duration enforced in code, and drops spawned from the enemy's death event, with the amount set by how it died.

### Numbers

- DOOM: a glory kill takes 600 to 700 ms from start to finish, the amount id found right. — [PC Gamer](https://www.pcgamer.com/doom-feels-like-bruce-lee-with-a-shotgun-on-a-skateboard-says-id/)
- DOOM: glory kills have "strict time limits" and last "hundreds of milliseconds", in Robert Duffy's words. — [Bethesda.net](https://bethesda.net/en-AU/news/the-guts-and-gore-of-doom-glory-kills)

### Pitfalls

- Regenerating health in a game about speed: the player stops to recover and the push is gone, which is why id dropped it.
- Long finishing animations: they break the flow they were meant to reward.

### References

- DOOM (2016): id Software at QuakeCon on movement, glory kills and dropping regenerating health. — [PC Gamer](https://www.pcgamer.com/doom-feels-like-bruce-lee-with-a-shotgun-on-a-skateboard-says-id/)
- DOOM (2016): Marty Stratton and Robert Duffy on where glory kills came from and how they work. — [Bethesda.net](https://bethesda.net/en-AU/news/the-guts-and-gore-of-doom-glory-kills)

## Melee combat in third person

Subgenres: action-3d, tps

### Expectations

- The player controls the camera, and the camera decides what the player can fight.
- Every attack lands where the player meant it to, with as much help as the difficulty allows.

### Systems

- Attacks steer towards where the camera faces: in God of War the melee attacks direct enemies towards the camera's facing, so framing the crowd is part of fighting it.
- Melee target assistance is a difficulty lever: on God of War's hardest setting Kratos no longer slides forward to meet his opponent with every strike.
- Lock-on and recentring the camera on attack help against one enemy or a small group and get in the way in a crowd, so they are options, not rules.
- Difficulty through behaviour, not only numbers: faster counter-attacks after a hit reaction or a block, and enemies that turn elite with new attacks and patterns.
- Stun as a meter apart from health, with more stun from hits in the back and from knocking enemies into walls.
- A sidestep keeps the combo going while a roll resets it, so repositioning has a cost the player chooses.
- In Unity, pick attack targets inside a cone around the camera's forward on the ground plane, and make the forward slide a short correction whose maximum distance comes from the difficulty setting.

### Numbers

- Santa Monica Studio describes these levers without publishing their values in this source. — [PlayStation Blog](https://blog.playstation.com/2018/05/16/16-tips-to-conquer-the-give-me-god-of-war-difficulty-mode/)

### Pitfalls

- Recentring the camera on every attack in a crowd: it fights the player's own framing.
- Assistance so generous that distance stops mattering: the hardest mode removes it precisely to make judging distance a skill.

### References

- God of War (2018): Mihir Sheth on the hardest difficulty and on how the camera drives combat. — [PlayStation Blog](https://blog.playstation.com/2018/05/16/16-tips-to-conquer-the-give-me-god-of-war-difficulty-mode/)
- Evolving Combat in God of War for a New Perspective (GDC 2019, Mihir Sheth). — [GDC Vault](https://gdcvault.com/play/1026423/Evolving-Combat-in-God-of)

## Aim assist

Subgenres: fps, tps

### Expectations

- With a controller, aiming feels precise without the game aiming for the player.
- A shot that looked like a hit is a hit, and a miss looks like a miss.

### Systems

- Two parts: stickiness, where the crosshair slows down as it passes over an enemy, and bullet magnetism, where a near miss still hits.
- Stickiness only for controllers: with a mouse it breaks the muscle memory aiming relies on.
- Bullet magnetism for the mouse too when fighting animated AI enemies; without it, shots that felt right register as misses and the game feels buggy.
- The assistance belongs to the weapon, not to the enemy's hitboxes. Witchfire uses nested circles around the crosshair, cones in 3D: two for stickiness, two for magnetism and one that favours critical areas such as the head.
- Each weapon has its own values, and the assistance grows up to the weapon's effective range and drops quickly past it.
- Trace the enemy's main bones or its capsule, not every limb, so a waving hand does not slow the crosshair.
- In Unity, cone checks around the camera's forward against each target's bounds, with values per weapon in a `ScriptableObject` and an `AnimationCurve` for strength over distance.

### Numbers

- The Astronauts publish the shape of their system, not its values: each weapon has its own set of circles. — [The Astronauts](https://www.theastronauts.com/2019/02/witchfire-aim-assist/)

### Pitfalls

- Bigger hitboxes instead of magnetism: shots get stopped by the invisible edge of cover.
- Bullets that curve towards the target: with any travel time, enemies behind cover start taking hits.
- Assistance strong enough to make headshots free: tuned right, it only removes the frustration of shots that felt right.

### References

- Witchfire: Adrian Chmielarz on how its aim assist works and why each part exists. — [The Astronauts](https://www.theastronauts.com/2019/02/witchfire-aim-assist/)

## Online shooting

Subgenres: fps, tps

### Expectations

- What the player sees is what the server judges: a hit on screen is a hit.
- Holding an angle is not a losing position by default.

### Systems

- The server is authoritative and simulates in fixed ticks; clients send input commands and receive snapshots of the world.
- Interpolation: clients draw other players slightly in the past, between two received snapshots, so movement stays smooth and one lost snapshot does not show.
- Prediction for the local player only, corrected when the server's answer differs.
- Lag compensation: the server rewinds other players to what the shooter saw, counting latency and interpolation, so players do not have to lead their shots.
- Peeker's advantage: whoever moves into view sees the other first, because the movement has to reach the server and then the defender. Higher tick rates, less buffering and faster clients shrink it.
- In Unity, choose a networking stack with server-authoritative prediction and lag compensation before writing gameplay; adding them later means rewriting the gameplay code.

### Numbers

- Source engine: a 15 ms tick, about 66 per second; by default clients receive about 20 snapshots and send about 30 command packets per second. — [Valve](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)
- Source engine: a 100 ms interpolation delay by default, so losing one snapshot still leaves two to interpolate between. — [Valve](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)
- VALORANT: in Riot's simplified model a peeker gets about 141 ms more to react; 128-tick servers, its own network and minimal buffering remove about 40 ms, and a client at 144 FPS brings it to about 71 ms. — [Riot Games](https://www.riotgames.com/en/news/peeking-valorants-netcode)
- VALORANT: in blind playtests skilled players noticed changes of about 10 ms, and 10 ms turned a 90 % win rate for an Operator holding an angle into 90 % for a rifle peeking. — [Riot Games](https://www.riotgames.com/en/news/peeking-valorants-netcode)

### Pitfalls

- Letting the client decide its own hits: a modified client can claim any hit it likes, which is why VALORANT resolves combat on the server.
- Large buffers to hide bad connections: every buffered frame adds latency for everyone.
- A low tick rate in a game where one shot can kill: every millisecond of delay goes to the peeker.

### References

- Source Multiplayer Networking (Valve): ticks, snapshots, interpolation, prediction and lag compensation. — [Valve Developer Community](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)
- Peeking into VALORANT's Netcode (Matt deWet and David Straily, Riot Games). — [Riot Games](https://www.riotgames.com/en/news/peeking-valorants-netcode)
