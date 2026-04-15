{{ coding_rules }}
---
{{ kpop }}

MANDATORY: Use the Task tool to launch up to 4 parallel subagents for:
- Exploring different areas of the codebase simultaneously
- Testing multiple hypotheses in parallel
Do NOT use sequential tool calls where parallel subagents would work.
---
KPop: Please review the codebase. Pay special attention to code that differs from branch main/.
- Is the code consistent with grounding.md and `{{ plan_path }}`?
- Is the code well-factored?
- Is the code idiomatic?
- Is the code elegant?

Write your review (problems only) to {{ review_path }}.

If everything is ok, just write
```
LGTM
```
in {{ review_path }}.
