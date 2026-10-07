# Accessibility

Sens should work for anyone who uses Claude Code: by keyboard, by voice, with a screen reader, without motion, in light or dark. This page says what Sens does today, what it does not do yet, and how to report a barrier. It describes what the code does. No test with a screen reader such as Narrator or NVDA has been recorded yet.

## Commitment

[The Sens visual identity](docs/brand/identity.md#36-contrast-and-accessibility) sets the bar every screen is held to: text at WCAG AA contrast, a visible focus ring, no state carried by colour alone, and motion that stops when Windows asks for less. A barrier is a bug, and is fixed like one.

## Supported environments

- Windows 10 and 11, x64. The interface runs in Microsoft Edge WebView2, which exposes it to Windows assistive technology as Edge exposes a web page.
- English, Español, Français, Deutsch, 日本語 and 简体中文. The page's language follows the one chosen, so a screen reader reads it with the right voice.

## What works

**Keyboard**

- Shortcuts for what is used most: `Ctrl+N` new session, `Ctrl+O` open a folder, `Ctrl+B` the sidebar, ``Ctrl+` `` the terminal, `Ctrl+,` settings, `Ctrl+Shift+L` the arrangement. *Profile › Keyboard shortcuts* lists them all.
- `Esc` closes a sheet or dialog, and focus returns to what opened it. It skips the welcome too, and while the welcome is open nothing behind it takes focus.
- Menus move with `↑` `↓`. Tabs — a panel's tools, its terminals, Settings, Capabilities — move with the arrow keys, `Home` and `End`, and `Tab` lands only on the one shown.
- Splitters resize with the arrow keys (`Shift` for larger steps). `Alt` with an arrow moves a tool to another panel.
- The Map's graph pans with the arrow keys, zooms with `+` and `-`, and fits with `0`, and says so to a screen reader; each area is picked with `Enter` or `Space`.
- In the floating bar, `Ctrl+Tab` switches project and `Tab` moves between its controls.
- In the message, `@` and `/` suggestions take `↑` `↓`, then `Enter` or `Tab`; the effort slider takes the arrow keys, `Home` and `End`.
- Every focused control shows a 2 px ring in the chosen accent; the floating bar's edge takes the accent while its field has focus.

**Screen readers**

- Every control has a name, in all six languages; icons, the mark and decorative motion are hidden from assistive technology.
- Menus, tabs, lists, switches, sliders, meters and progress bars carry their roles. Errors are announced as alerts and changes of state as status.
- Sens says when Claude needs an answer, when a reply has finished and what went wrong, without reading a reply aloud as it streams.
- Tool steps and thoughts are native disclosure elements, so they say whether they are open, and a step says its state in words: in progress, done, failed or stopped.

**Seeing**

- Dark, light or as Windows, in five accents.
- In the light theme every text role keeps at least 4.5:1 on every surface, for all five accents. The terminal's colours keep 4.5:1 in both modes. [The brand tests](test/brand.test.ts) compute these ratios on every change.
- State is never colour alone where it matters most: a failed step opens on its error, a stopped step is drawn as a ring, changed files carry their letter, diff lines their `+` and `−`, and permission outcomes are said in words. A test holds these in place.

**Motion**

- With animation effects turned off in Windows, transitions stop, animations stop or hold still — the empty chat's greeting and the welcome appear without fading in — the stone, the grain and the effort pixels draw one still frame, the terminal cursor stops blinking, and the installer shows its percentage without counting up to it.

**Voice**

- Dictation writes what you say into the message, with Whisper running on your computer. "Hey Sens", switched on in *Settings › Focus & voice*, opens focus mode already dictating, from any window.

**Installer**

- Every screen puts focus on its main button. Choices are native radios and checkboxes; progress, steps and errors are announced.

## Known limitations

**Screen readers**

- The text of a reply is not read aloud when it arrives, only that it finished; it is read by moving to it.
- The file tree is a list of buttons with their open state, not a tree.
- The terminal does not turn on xterm's screen reader mode.

**Keyboard**

- Some actions have no key: moving one area of the Map's graph, resetting a splitter (double-click), and moving the floating bar.

**Seeing**

- There is no zoom and no text size setting: `Ctrl` with `+` or `-` does nothing, and sizes are fixed. The installer is a fixed 880 × 520 window.
- Windows contrast themes are not followed; Sens keeps its own colours.
- Contrast is computed for the light theme and the terminal only; the dark theme's text is not checked by a test.
- A few states still rely on colour: files Claude touched in the tree, a session that waits against one that finished (a screen reader hears the difference; the eye sees only the colour), and files git ignores.

**Motion**

- The stone, the grain, the effort pixels and the terminal cursor read the setting when they appear; changing it while Sens is open takes effect the next time they do.

## Reporting a barrier

[Open an accessibility issue](https://github.com/beyondhumane/sens-ai/issues/new?template=barrier.yml). Say what you were trying to do, where Sens stopped you, and what you use — a screen reader and its version, keyboard only, voice, a contrast theme, magnification, reduced motion. Nothing about your setup is required beyond what helps reproduce the barrier.
