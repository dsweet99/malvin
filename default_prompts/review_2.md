{{ coding_rules }}
---
{{ kpop }}

MANDATORY: Use the Task tool to launch up to 3 parallel subagents for:
- Exploring different areas of the codebase simultaneously
- Testing multiple hypotheses in parallel
Do NOT use sequential tool calls where parallel subagents would work.
---
KPop: Please review the codebase. Pay special attention to code that differs from branch main/.
- Does the code **violate** any user contract stated in `grounding.md`? `grounding.md` is the user contract and may describe a **subset** of current behavior. Code that has **added** capability beyond `grounding.md`'s description is **not** a Problem; if you want to record it, put it under a `## Notes` section of {{ review_path }} — never under `## Problems`.
- Is the code consistent with `{{ plan_path }}`?
- Is the code well-factored?
- Is the code idiomatic?
- Is the code elegant?

Write your review (problems only) to {{ review_path }}. Informational drift between code and `grounding.md` belongs under `## Notes`, not `## Problems`.

If everything is ok, just write LGTM in {{ review_path }}.
