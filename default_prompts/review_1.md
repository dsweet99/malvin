{{ coding_rules }}
---
{{ kpop }}

Prefer parallel investigation. Launch up to 3 parallel subagents to:
- Explore different areas of the codebase simultaneously
- Test multiple hypotheses in parallel
Do NOT choose sequential execution when a parallel option is available.
---
KPop: Please review the codebase. Pay special attention to code that differs from branch main/.
- Is the code consistent with `{{ grounding_path }}` (if it exists and is not empty) and `{{ plan_path }}` (if it exists and is not empty)?
- Are there bugs?
- Is there avoidable redundancy or wasted work?
- Is the code well-tested?

For every bug you find, write a failing regression test that exposes it.

Write your review (problems only) to {{ review_path }}.

If everything is ok, just write LGTM in {{ review_path }}.
