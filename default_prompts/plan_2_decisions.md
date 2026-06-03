# Plan Prompt 2 — research and decide

You are running **Prompt 2** of `malvin plan`. Read `{{ plan_path }}`, especially `## Open questions`.

Research answers using the codebase, malvin logs, and external sources where appropriate. Make best-effort decisions on remaining questions; favor correctness; cite evidence (code path, log id, URL).

Append section `## DECISIONS` to `{{ plan_path }}` **immediately after** `## Open questions`. Each entry:

- Matches an open-question number (or `0` when there were zero open questions — document that no numbered questions existed and record one deliberate attempt to falsify an implicit plan assumption).
- States a verdict.
- Cites evidence.

Do **not** rewrite the plan body. Do **not** remove prior sections.

Write complete sentences. Use the file editing tools to append to `{{ plan_path }}`.
