{{ coding_rules }}
---
{{ kpop }}

MANDATORY: Prefer parallel investigation. If the Task tool is available, launch up to 3 parallel subagents to:
- Exploring different areas of the codebase simultaneously
- Testing multiple hypotheses in parallel
If the Task tool is unavailable, use parallel non-Task tools where possible; otherwise proceed sequentially and explicitly note the fallback and any resulting coverage limits.
Do NOT choose sequential execution when a parallel option is available.
---
KPop: Please review the codebase. Pay special attention to code that differs from branch main/.
- Is the code consistent with grounding.md (if it exists and is not empty) and `{{ plan_path }}` (if it exists and is not empty)?
- Is the code well-factored?
- Is the code idiomatic?
- Is the code elegant?

Write your review (problems only) to {{ review_path }}.

If everything is ok, just write LGTM in {{ review_path }}.
