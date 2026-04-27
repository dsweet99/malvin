{{ coding_rules }}
---
{{ kpop }}

Prefer parallel investigation. Launch up to 3 parallel subagents to:
- Explore different areas of the codebase simultaneously
- Test multiple hypotheses in parallel
Do NOT choose sequential execution when a parallel option is available.
---
KPop: Please review the codebase. Pay special attention to code that differs from branch main/.
- Is the code consistent with grounding.md (if it exists and is not empty) and `{{ plan_path }}` (if it exists and is not empty)?
- Is the code well-factored?
- Is the code idiomatic?
- Is the code elegant?

Write your review (problems only) to {{ review_path }}.

If everything is ok, just write LGTM in {{ review_path }}.
