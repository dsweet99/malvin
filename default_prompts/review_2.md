{{ coding_rules }}
---
{{ kpop }}

Prefer parallel investigation. Launch up to 3 parallel subagents to:
- Explore different areas of the codebase simultaneously
- Test multiple hypotheses in parallel
Do NOT choose sequential execution when a parallel option is available.

---

KPop: Please review the codebase. Pay special attention to code that differs from branch main/.
- Is the code consistent with `{{ plan_path }}` when a plan file exists for this run, and with the Quality Gates section?
- Is the code well-factored?
- Is the code idiomatic?
- Is the code elegant?

Write your review (problems only) to {{ review_path }}.

If everything is ok, write *only* and *exactly* LGTM in {{ review_path }}.
