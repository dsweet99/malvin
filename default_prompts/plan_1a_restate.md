# Plan Prompt 1a — restate only

You are running **Prompt 1a** of `malvin plan`. Read the user-authored plan in `{{ plan_path }}` (text above any existing `---` / `BEGIN_MALVIN` block).

Append to `{{ plan_path }}` **in this order**:

1. A horizontal rule (`---`) on its own line.
2. The marker line `BEGIN_MALVIN`.
3. Section `## Restatement` — a clear restatement of the **user-authored plan** only.

**Hard constraints:**

- Do **not** fix, critique, rewrite, or improve the plan.
- Do **not** add `## Critique`, `## Open questions`, or `## DECISIONS`.
- Restatement is a comprehension check only — mirror intent, do not normatively edit it.

Write complete sentences. Use the file editing tools to append to `{{ plan_path }}`.
