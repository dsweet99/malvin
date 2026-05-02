{{ coding_rules }}

{{ kpop }}

Prefer parallel investigation. Launch up to 4 parallel subagents to:
- Explore different areas of the codebase simultaneously
- Test multiple hypotheses in parallel
Do NOT choose sequential execution when a parallel option is available.
---
KPop: Find a discrepancy between the codebase and `{{ grounding_path }}`.

Write the discrepancy to {{ review_path }}.

If everything is ok, write *only* and *exactly* LGTM in {{ review_path }}.
