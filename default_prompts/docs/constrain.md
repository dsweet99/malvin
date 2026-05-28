# malvin constrain

Write a regression test for a plan, then implement code so the test passes — using the same kpop gate-loop workflow as `malvin code`, with `constrain_constraints.md` as the scope prompt.

## Intention

Take a written plan (inline or from a file) and drive the repo through a KPop gate session focused on test-first constraint satisfaction.

## Usage

```text
malvin constrain [OPTIONS] <REQUEST>
```

## Arguments

### `<REQUEST>` (required)

Plan text or a path to an existing `.md` file (no whitespace in the path string; case-sensitive `.md` suffix). Copy stored as `plan.md` in the run directory. Nonexistent `.md` paths are treated as literal text.

## Options

Same gate-loop options as `malvin code`: `--max-loops`, `--max-hypotheses`, `--tenacious`, and global malvin flags. See `code.md` for details.

## Requirements

- `kiss` on PATH (CLI refuses to start otherwise)
- Cursor agent CLI
- Quality gates from `.malvin/checks`

## Prompt workflow

Same kpop multiturn gate-loop as `malvin code`, but scope constraints come from `constrain_constraints.md` (regression test first, then implementation).

## Examples

```text
malvin constrain plan.md
malvin constrain "Add regression test for widget bug"
```
