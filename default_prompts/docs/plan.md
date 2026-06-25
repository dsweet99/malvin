# malvin plan

Four **sequential** agent prompts on **one persistent session** to reflect on a written plan, resolve uncertainty with cited evidence, and emit a revised implementation plan.

## Summary

| | |
|---|---|
| Input | Existing `.md` plan file |
| Session | One persistent agent session (Prompt 1a → 1b → 2 → 3) |
| Output | In-place overwrite of `PLAN_PATH` after Prompt 3 |
| Downstream | Post-Prompt-3 file is valid input for `malvin code <PLAN_PATH>` |

## Usage

```text
malvin plan [OPTIONS] <PLAN_PATH>
```

## Arguments

### `<PLAN_PATH>` (required)

Existing `.md` file (case-sensitive suffix, no whitespace in path).

## File shape after Prompt 3

```text
[revised implementation plan — normative spec only]
```

After Prompt 3, `PLAN_PATH` contains only the revised implementation plan. This file is valid input for `malvin code`.

Intermediate sections (restatement, critique, open questions, decisions) live in run-dir snapshots (`plan.p1a.md`, `plan.p1b.md`, `plan.p2.decisions.md`), not in the final plan file.

Before Prompt 1a, malvin atomically overwrites `PLAN_PATH` with `## Restatement` and preserves the user plan in a sidecar (`PLAN_PATH.malvin-user`) plus a run-dir copy (`plan.md`). The agent fills in restatement prose only; it must not add or move the `## Restatement` heading.

## Re-run

If the plan file still contains intermediate machine sections from an interrupted run (`## Restatement` at the top), malvin restores the user plan from the sidecar (or, for legacy runs, truncates before a `---` / `BEGIN_MALVIN` block) before Prompt 1a. After a successful Prompt 3 overwrite, re-running treats the entire file as the user-authored input for a fresh pass.

## Adversarial profile

When the plan path matches `*adversarial*` or `*adv_system*`, or `smell_registry.toml` exists in the work directory, Prompt 1b and Prompt 3 include extra obligations (smell-registry mapping, MR/PBT classes, materialization harness milestones).

## Related commands

| Command | When |
|---------|------|
| `malvin code <PLAN_PATH>` | Implement the revised plan |
| `malvin inspire` | Creative exploration before planning |
| `malvin kpop` | Hypothesis-driven investigation |

## Examples

```text
malvin plan plan.md
malvin plan docs/feature_plan.md
```
