# Plan Prompt 1a — restate only

You are running **Prompt 1a** of `malvin plan`. Read the user-authored plan in `{{ plan_path }}` (content above the `---` / `BEGIN_MALVIN` block that malvin already appended at the end of the file).

Edit `{{ plan_path }}` by writing your restatement **only** under the existing `## Restatement` heading (immediately below `BEGIN_MALVIN`). Do **not** add, move, duplicate, or edit the `---` line, the `BEGIN_MALVIN` line, or any text above `BEGIN_MALVIN`.

**Hard constraints:**

- Do **not** fix, critique, rewrite, or improve the plan.
- Do **not** add `## Critique`, `## Open questions`, or `## DECISIONS`.
- Do **not** include the literal text `BEGIN_MALVIN` in your restatement prose.
- Ignore every standalone `---` line in the user plan; they are markdown section dividers, not machine boundaries.
- Restatement is a comprehension check only — mirror intent, do not normatively edit it.

Write complete sentences. Use the file editing tools to edit `{{ plan_path }}`.
