# malvin plan

Four **sequential** agent prompts on **one persistent session** to reflect on a written plan, resolve uncertainty with cited evidence, and emit a revised implementation plan.

## Summary

| | |
|---|---|
| Input | Existing `.md` plan file |
| Session | One persistent agent session (Prompt 1a → 1b → 2 → 3) |
| Output | In-place update of `PLAN_PATH` after Prompt 3 splice |
| Downstream | Post-Prompt-3 file is valid input for `malvin code @PLAN_PATH` |

## Usage

```text
malvin plan [OPTIONS] <PLAN_PATH>
```

## Arguments

### `<PLAN_PATH>` (required)

Existing `.md` file (case-sensitive suffix, no whitespace in path).

## File shape after Prompt 3

```text
[user-authored plan]

---
BEGIN_MALVIN
[revised implementation plan — normative spec only]
```

Content above the first machine `---` is user context. Content below `BEGIN_MALVIN` is the implementation plan for `malvin code`.

Intermediate sections (restatement, critique, open questions, decisions) live in run-dir snapshots (`plan.p1a.md`, `plan.p1b.md`, `plan.p2.decisions.md`), not in the final plan file.

Before Prompt 1a, malvin atomically appends `\n---\nBEGIN_MALVIN\n## Restatement\n` to the plan file. The agent fills in restatement prose only; it must not add or move the delimiter lines.

## Re-run

Re-running replaces the machine block (from the first `---` / `BEGIN_MALVIN` through EOF) with a fresh pass. User text above the machine block is preserved.

## Adversarial profile

When the plan path matches `*adversarial*` or `*adv_system*`, or `smell_registry.toml` exists in the work directory, Prompt 1b and Prompt 3 include extra obligations (smell-registry mapping, MR/PBT classes, materialization harness milestones).

## Related commands

| Command | When |
|---------|------|
| `malvin code @PLAN_PATH` | Implement the revised plan |
| `malvin inspire` | Creative exploration before planning |
| `malvin kpop` | Hypothesis-driven investigation |

## Examples

```text
malvin plan plan.md
malvin plan docs/feature_plan.md
```
