# malvin init

Discover how the workspace runs quality gates today and write `.malvin/checks` (one shell command per non-empty line).

## Summary

| | |
|---|---|
| Input | None |
| Prompt | `init_constraints.md` |
| Agent | KPop checks-discovery session when `.malvin/checks` is missing |
| Fast path | If `.malvin/checks` already exists (including empty or comment-only), **no agent** |
| Requires | Cursor agent CLI only when discovery runs |

## Intention

Bootstrap a repo for gated workflows (`malvin tidy`, bare `malvin REQUEST --gates`) without running those workflows. Use this when you want `.malvin/checks` materialized explicitly.

## Usage

```text
malvin init [OPTIONS]
```

No positional arguments. Work directory is always `.` (cwd).

## Behavior

1. If `.malvin/checks` is missing, malvin runs a KPop session scoped by `init_constraints.md`. The agent inspects existing repo tooling (for example `.pre-commit-config.yaml`, `Makefile`, CI workflows) and writes `.malvin/checks` at the repo git root.
2. If `.malvin/checks` is present after discovery — including an empty or comment-only file with zero runnable commands — malvin exits successfully. A missing file still fails.
3. If `.malvin/checks` already exists, malvin exits successfully without spawning an agent.

Delete `.malvin/checks` to trigger discovery again.

## Options

Inherits global malvin options (`--model`, `--no-force`, `--verbose`, etc.). No init-specific flags.

## Notes

- Discovery uses repo signals only; malvin does not invent default linters or test runners when the repo provides no signal.
- Comment lines in `.malvin/checks` start with `#` after trimming and are ignored when running gates.
