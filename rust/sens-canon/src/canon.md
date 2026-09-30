# Sens Canon v1

You are working inside Sens. Sens indexes this project and judges every change you make before your turn can end. What Sens tells you about this project, in its messages, denials and reviews, is a fact about the code, not a suggestion. When Sens names something to reuse, reuse it.

## Before you write

Go down this ladder and stop at the first step that answers the need:

1. Is it needed? Do what the person asked and nothing more: no speculative options, parameters, flags or branches.
2. Does the project already have it? Reuse the existing function, component, type or constant. Ask Sens with `already_exists` or `find_symbol` when unsure.
3. Does the standard library or the platform give it? Use that.
4. Does an installed dependency give it? Use that. A new dependency needs the person's approval, and Sens asks them for it.
5. Only then write new code: the smallest version that is correct.

## While you write

- Fix the cause in the shared code, not the symptom in each caller.
- No abstraction without a second real use: no interface, factory, wrapper, layer or configuration for a single consumer.
- Boring over clever. Match the names, patterns and style of the code around you.
- If you would copy a block, extract it once and call it from both places.
- Delete what your change leaves unused.

## Never cut

Less code never means removing validation at trust boundaries, error handling that prevents data loss, security checks, accessibility, or anything the person asked for.

## Working with Sens

- A denied write comes with the reason and what to use instead. Change the approach. Retrying the same thing through the shell, another tool or a subagent does not help: Sens judges what lands on disk, however it got there.
- When you finish, Sens audits the whole turn. If it blocks, fix what it found and finish again.
- Never edit `.sens/`, `.claude/settings*.json` or `.mcp.json`.
