{{ kpop }}
---
KPop: Check {{ plan_path }} for blocking issues. This is a READ-ONLY review — do NOT edit source code, do NOT run tests, do NOT implement anything. Falsify by reading code, not by writing it.

A plan is acceptable if it:
- Does NOT contradict the Quality Gates section (see reference below) and other hard requirements given in this session (silence on details is fine when those sources do not speak to a topic)
- Is internally consistent (no contradictory requirements)
- Refers to files and APIs that exist in the codebase (you may read files to verify)

Brief plans like "Write this app" or "Implement the feature" are valid when memories and coding rules supply enough context. The plan does not need to restate every quality-gate bullet.

If plan is acceptable, write ONLY the four characters "LGTM" to {{ review_path }}. No explanation, no additional text—just LGTM.

If there IS a blocking issue, write a brief explanation to {{ review_path }} (without LGTM).

You have a budget of 4 hypotheses.

## Reference
The coding rules below are what the coding agent that implements this plan will see. Your task is *not* to implement the plan. The rules here are for your reference only. They may help you better evaluate the plan.

BEING_CODING_RULES
```
{{ coding_rules }}
```
END_CODING_RULES
