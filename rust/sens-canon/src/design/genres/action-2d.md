# Action 2D and platformers

Numbers from Celeste are pixels on its 320×180 canvas, where a tile is 8 pixels. Carry them to another game through tiles and seconds, never as raw pixels.

## Run

### Expectations

- The character reaches full speed almost at once and stops almost at once, so every input shows in the next frames.
- In the air control is weaker but never gone: the player can still steer a jump.

### Systems

- Horizontal speed approaches the target speed (input × maximum) by a fixed amount per second. Celeste does this with one `Calc.Approach` call for accelerating and stopping alike.
- A separate, gentler reduction applies only above the maximum while the stick still points the same way, so dashes and boosts keep their momentum instead of being cut.
- In Unity, move a kinematic body or a controller built on `Physics2D` casts with your own velocity. A dynamic `Rigidbody2D` that stops through material friction stops differently on every surface.

### Numbers

- Celeste: maximum run speed 90 px/s, about 11 tiles per second. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L30)
- Celeste: accelerates and stops at 1000 px/s², so it reaches full speed or a standstill in under a tenth of a second. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L2894)
- Celeste: above the maximum, while the stick still points the same way, it slows at only 400 px/s², so extra speed fades rather than snaps; letting go stops it at the full 1000. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L2891)
- Celeste: in the air both rates are multiplied by 0.65. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L33)
- Celeste's tile is 8 pixels: two pixels are "a quarter of a tile", in its designer's words. — [Celeste & Forgiveness](https://maddymakesgames.com/articles/celeste_and_forgiveness/)

### Pitfalls

- Stopping through physics friction: the stopping distance changes with the floor and the frame rate, and the character slides on slopes.
- Long acceleration ramps by default: precise platforming turns floaty and unfair. A ramp belongs to a heavy character by design.
- Cutting all speed above the maximum: dashes and moving platforms lose their momentum and feel dead.

### References

- Celeste, the running and friction block of `Player.cs`: one function handles the ground, the air and over-speed.

## Jump

### Expectations

- Holding the button jumps higher and tapping it jumps lower.
- A jump pressed a moment after leaving a ledge still happens, and one pressed a moment before landing fires on the landing frame.
- The top of the jump hangs briefly, so the player can aim the landing.

### Systems

- Design the jump from a height and a time to the apex and derive the launch speed and gravity from them, instead of tuning both by hand. That is the method of *Building a Better Jump*.
- Variable height: while the button stays down during a short window, the upward speed is held; releasing ends the window and gravity takes over.
- Coyote time: a timer refilled while grounded; a jump is allowed while it runs.
- Jump buffer: a press is remembered for a short time and consumed on landing.
- Apex: gravity is halved near the top while the button is held.
- Fast fall: holding down raises the maximum fall speed gradually, and the sprite stretches to show it.
- In Unity, read the press every frame in `Update` and apply it in the physics step. A press read only in `FixedUpdate` can be lost between steps.

### Numbers

- Celeste: coyote time 0.1 s. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L48)
- Celeste: a jump launches at 105 px/s upward and adds 40 px/s sideways in the held direction. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L1670)
- Celeste: the upward speed is held for up to 0.2 s while the button is down. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L2961)
- Celeste: gravity 900 px/s², halved while the button is held and the vertical speed is under 40 px/s. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L2952)
- Celeste: maximum fall 160 px/s; holding down once already falling that fast raises it towards 240 px/s at 300 px/s². — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L2910)
- Celeste's buffer length is not in its published code; its designer only says "a short time before landing". — [Celeste & Forgiveness](https://maddymakesgames.com/articles/celeste_and_forgiveness/)

### Pitfalls

- The same gravity up and down and no variable height: the jump is a fixed arc the player cannot shape.
- Coyote or buffer windows long enough to notice: the player feels the game jumping on its own.
- A jump designed by tweaking gravity and speed separately: every change to one breaks the height or the timing.

### References

- Celeste & Forgiveness: coyote time, buffering and the halved-gravity peak, explained by the designer who added them.
- Building a Better Jump (GDC 2016, Kyle Pittman): trajectories from a height and a distance, variable height and falling faster after the peak. — [GDC Vault](https://www.gdcvault.com/play/1023148/)

## Camera

### Expectations

- The player sees more of where they are going than of where they have been.
- The camera does not follow every small hop.

### Systems

- Camera window: the camera moves only when the player pushes against the edge of a box around them.
- Platform snapping: the camera follows vertically when the player lands, not during the jump.
- Forward focus: extra view in the direction of travel. With dual forward focus the camera keeps its side until the player moves past a threshold the other way.
- Lerp smoothing: each frame the camera closes a fraction of the distance to its target.
- Manual focus through controls that already exist: looking up or crouching shifts the view.
- In Unity, move the camera in `LateUpdate`, after the player has moved that frame, and interpolate the player's `Rigidbody2D` if it moves in the physics step; otherwise the image jitters.

### Numbers

- No number holds across games: the window and the smoothing depend on the character's speed and the screen, so the reference documents the technique per game. — [Scroll Back](https://www.gamedeveloper.com/design/scroll-back-the-theory-and-practice-of-cameras-in-side-scrollers)

### Pitfalls

- A camera locked to the player's position: every jump shakes the whole screen.
- Smoothing alone with a very fast character: it reaches the edge of the screen before the camera catches up.
- A camera that turns around on every small change of direction.

### References

- Super Mario World: platform snapping, and a threshold before the forward focus switches sides.
- Sonic the Hedgehog: a narrow, tall window keeps a fast character near the centre.
- Spelunky: looking up and crouching move the focus.
- Scroll Back (Itay Keren, GDC 2015): the whole catalogue of side-scroller cameras, with a glossary. — [Game Developer](https://www.gamedeveloper.com/design/scroll-back-the-theory-and-practice-of-cameras-in-side-scrollers)

## Collision and forgiveness

### Expectations

- Grazing a corner does not stop a jump or a dash.
- Demanding moves get wider windows than easy ones.

### Systems

- Corner correction: when the head hits a solid on the way up, try shifting a few pixels sideways and carry on if that space is free.
- Dash corner correction: a sideways dash that clips a ledge pops the character up onto it.
- Wall jumps checked a few pixels away from the wall, not only on contact.
- Grace windows that turn one action into another after the fact, like Celeste's stamina refund.
- In Unity, these are extra `BoxCast` or `OverlapBox` checks in the controller before a collision is resolved; physics materials cannot express them.

### Numbers

- Celeste: up to 4 px of upward corner correction, half a tile. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L53)
- Celeste: up to 4 px of dash corner correction. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L82)
- Celeste: wall jumps from 2 px away, a quarter of a tile; super wall jumps from about 5 px, by its designer's own estimate. — [Celeste & Forgiveness](https://maddymakesgames.com/articles/celeste_and_forgiveness/)

### Pitfalls

- Corrections the player can see: past half a tile they stop feeling like forgiveness and start feeling like the game moving the character.
- Testing only the intended path: players find the corners by running into them at full speed.

### References

- Celeste & Forgiveness: ten small windows, every one widened in the player's favour.

## Dash

### Expectations

- A fixed distance, a fixed duration and an instant start, so the player can predict where it ends.
- A short freeze at the start sells the burst.

### Systems

- A dash is a state: set the velocity to direction × dash speed, ignore gravity while it lasts, then hand over a lower exit speed instead of zero.
- A cooldown that is separate from whatever refills the dash.
- In Unity, a state in the controller's state machine, not a force added to a dynamic body.

### Numbers

- Celeste: 240 px/s for 0.15 s, about 4.5 tiles. A sideways or upward dash then drops to 160 px/s, with its upward part cut to three quarters; a downward dash keeps its speed. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L3625)
- Celeste: a new dash cannot start until 0.2 s after the previous one began. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L3381)
- Celeste: the game freezes for 0.05 s when a dash begins. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L3449)

### Pitfalls

- Speed cut to zero when the dash ends: it stops dead and breaks the flow into the next move.
- A dash affected by gravity or friction: its length changes with the situation, so it cannot be planned.

### References

- Celeste, `DashBegin` and the dash routine in `Player.cs`.

## Combat feedback

Subgenres: metroidvania, action-2d

### Expectations

- Every hit is felt: a freeze, a reaction and a sound.
- Being hit is readable and survivable: the player sees what hit them and gets a moment to recover.

### Systems

- Hitstop: attacker and target freeze for the same time, longer for stronger hits, shorter for projectiles, and never past a cap.
- During the freeze the target shakes with a shrinking amplitude and the attacker keeps moving very slightly: small touches that, in Sakurai's words, make the impact feel a little stronger each.
- With many enemies, keep the freeze very short or freeze only the two entities involved; a long global freeze on every hit slows the whole game.
- Knockback is a velocity impulse away from the hit with a short loss of control. Invulnerability after being hit is shown, usually by blinking.
- In Unity, freeze one entity by pausing its animator and its controller. `Time.timeScale = 0` stops everything, physics included.

### Numbers

- Super Smash Bros.: during hitstop the hurt pose is reached over 4 frames, so it does not snap. — [Sakurai, Famitsu column](https://sourcegaming.info/2015/11/11/thoughts-on-hitstop-sakurais-famitsu-column-vol-490-1/)
- Celeste: a 0.05 s freeze when a dash starts and 0.1 s when an explosion launches the player. — [Player.cs](https://github.com/NoelFB/Celeste/blob/master/Source/Player/Player.cs#L3985)

Knockback distance and invulnerability length have no published value in these sources; tune them against the rhythm of the enemies' attacks.

### Pitfalls

- Hitstop longer than the game's rhythm allows: in a crowd, a frozen pair is an open target, which is why Smash keeps it shorter than one-on-one fighters.
- Invulnerability with no visible signal: the player cannot tell why a hit did nothing.

### References

- Thinking About Hitstop (Masahiro Sakurai, Famitsu, translated): how Smash scales, caps and dresses the freeze.

## Metroidvania world

Subgenres: metroidvania

### Expectations

- The world is one coherent space the player can map in their head.
- Leaving the main path pays off.
- New abilities reopen old places.

### Systems

- Settle the order of the abilities together with the rough shape of the world, then trace the path through it to each ability; loosen the hard gates afterwards.
- Connect areas wherever it makes sense. A door that leads somewhere spatially impossible breaks the map in the player's head.
- Sequence breaks stay in unless the player can get stuck without noticing.
- In Unity, areas as scenes loaded additively, and gates as data (which ability opens what) in `ScriptableObject` assets, so progression can be checked without playing it.

### Numbers

- Building Hollow Knight's map "wasn't really an analytical process", in the words of its co-director; there is no formula to copy. — [PC Gamer](https://www.pcgamer.com/how-to-design-a-great-metroidvania-map/)

### Pitfalls

- Gates so hard the world becomes a corridor: Team Cherry removed many until a lot of the abilities were optional.
- Asking the player to explore and leaving detours empty: every one needs something to find.
- A dead end a few steps from where the player wants to be.

### References

- Hollow Knight: Team Cherry on how the map was built, from the first sketches to the final world. — [PC Gamer](https://www.pcgamer.com/how-to-design-a-great-metroidvania-map/)

## Level design

### Expectations

- Each level is about one idea, and teaches it without text.

### Systems

- Kishōtenketsu: introduce the idea, develop it a little more, change it up, then conclude.
- One clear concept per level, kept through the whole level.
- Playtest with people outside the team.
- In Unity, build levels from prefabs that carry the mechanic, so changing one tuning value changes every instance that does not override it.

### Numbers

- Hayashida structures a level in four steps, after the four panels of a Japanese comic strip. — [Koichi Hayashida, Game Developer](https://www.gamedeveloper.com/design/the-structure-of-fun-learning-from-i-super-mario-3d-land-i-s-director)

### Pitfalls

- Teaching with text instead of a safe first encounter.
- A twist before the idea has been developed: the player blames the controls.
- Several new ideas in one level: none of them gets developed.

### References

- Super Mario 3D Land and Super Mario Galaxy 2: their director on structuring levels like a four-panel comic.
