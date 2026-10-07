# Contributing to Sens

Issues and pull requests are welcome. This page says where to start, which rules keep the codebase the way it is, and what a pull request needs to be merged.

Everyone who takes part follows the [code of conduct](CODE_OF_CONDUCT.md).

## Where to start

- **A bug.** Open an issue with the bug form, with the Sens version and the steps that show it.
- **An idea, or a change larger than a fix.** Open an issue first, so the design is agreed before the code is written. The designs behind what exists are in [`docs/specs`](docs/specs).
- **An accessibility barrier.** Open an issue with the accessibility form; [Accessibility](ACCESSIBILITY.md) lists the barriers already known.
- **A vulnerability.** Not an issue: follow the [security policy](SECURITY.md).
- **A problem with Claude Code itself**, not with the window around it, belongs in [Claude Code's issues](https://github.com/anthropics/claude-code/issues).

## Setting up

[Build from source](README.md#build-from-source) lists what to install. Then:

```bash
npm ci
npm run app:dev
```

Work on the interface alone needs no Rust: `npm run dev -w sens-app-ui` serves it to any browser over a simulated Tauri, as [the README](README.md#the-interface-without-rust) describes.

## Rules

- **No comments.** None, in any file: not `//`, `/* */`, `///`, `#` or `<!-- -->`, not doc comments on a public API, not a TODO meant to be removed later. Code explains itself with names and structure; when something needs explaining, extract a function whose name says it. A file you touch that still has a comment loses it. The one exception is a `#!` line an executable needs to run.
- **Reuse before writing.** Look for what already does the job before adding a function, component, type or constant, and never write a second version of it. The best change often removes more than it adds.
- **Six languages.** Anything a person reads exists in English, Español, Français, Deutsch, 日本語 and 简体中文, `aria-label` and `title` included. `copy()` refuses to compile when a language is missing a key; [Sens in six languages](docs/i18n.md) has the voice and the glossary. Tests run in Spanish.
- **One identity.** Anything visual follows [the Sens visual identity](docs/brand/identity.md); read it before proposing a visual change. Colours live in `src/brand/tokens.ts` and the mark's geometry in `src/brand/mark.ts`: import the token instead of retyping a hex, and change the generator (`npm run brand`), not the asset it wrote. Icons come from Lucide, as SVG.
- **Commits say what changed for the person using Sens.** `fix(app): Enter while Claude works keeps the message`, not `fix: handle keydown`. The type is `feat`, `fix`, `refactor`, `docs`, `ci` or `chore`; the scope is the area, such as `app`, `map`, `canon`, `index` or `web`.
- **Leave the version alone.** Releases are cut separately, with `chore(release)`.

## Checks

Run what CI runs before you open a pull request: [Tests](README.md#tests) lists it. CI runs it again on every push, and a pull request is merged only when it is green and clippy has nothing to say. A change under `web/` is also checked with `npm run check` inside `web/`.

## Pull requests

- Branch from `main`, named for the work: `feat/…`, `fix/…`, `docs/…`.
- One subject per pull request. A refactor that a feature needs can travel with it; one it does not need goes on its own.
- The template asks three things: why, what, and what you checked. Say what you could not check as plainly as what you did.
- A change someone can see comes with screenshots, in light and dark.
