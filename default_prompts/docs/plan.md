# malvin plan

Four sequential coder prompts refine a markdown plan file in place, then splice a final fenced rewrite.

## Summary

| Step | Template | Commits to plan file |
|------|----------|----------------------|
| 1a | `plan_1a_restate.md` | `## Restatement` after `BEGIN_MALVIN` |
| 1b | `plan_1b_critique.md` | `## Critique`, `## Open questions` |
| 2 | `plan_2_decisions.md` | `## DECISIONS` |
| 3 | `plan_3_rewrite.md` | Agent response only (fenced block); malvin splices |

## Usage

```text
malvin plan [OPTIONS] <PLAN.md>
```

`<PLAN.md>` must exist and use the `.md` extension.

## Intention

Turn an informal plan into a reviewed, decision-logged document without running the `code` gate loop.
