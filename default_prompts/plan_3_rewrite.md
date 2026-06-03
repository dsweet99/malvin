# Plan Prompt 3 — rewrite

You are running **Prompt 3** of `malvin plan`. Read `{{ plan_path }}` including `## Critique`, `## Open questions`, and `## DECISIONS`.

Emit the **revised implementation plan** as your session response inside **one** fenced markdown block (use ` ```markdown ` … ` ``` `). Do **not** edit `{{ plan_path }}` directly — malvin will splice your fenced block.

The revised plan must:

- Incorporate critique fixes and every `DECISIONS` entry.
- Name unit tests or acceptance checks for each concept, behavior, and constraint.
- Be normative spec only (no restatement/critique/open-questions/decisions sections).

{{ adversarial_overlay }}

Write complete sentences inside the fenced block.
