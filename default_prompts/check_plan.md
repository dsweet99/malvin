{{ coding_rules }}
---
{{ kpop }}
---
KPop: Check {{ plan_path }} for blocking issues.

A plan is acceptable if it:
- Does NOT contradict grounding.md (silence on details is fine—grounding.md fills gaps)
- Is internally consistent (no contradictory requirements)
- Is feasible given the codebase

Brief plans like "Write this app" or "Implement the feature" are valid when grounding.md provides context. The plan does not need to restate grounding.md requirements.

If acceptable, write ONLY the four characters "LGTM" to {{ review_path }}. No explanation, no additional text—just LGTM.

If there IS a blocking issue, write a brief explanation to {{ review_path }} (without LGTM).
