
{{ kpop }}

---

Don't change code.

## Step 1: (Maybe) Restate plan

If the plan in {{ plan_path }} is brief or simplistic, then:
- Restate the user's request clearly, writing to {{ plan_path }}. Append any questions you might have to the end of the file.
- Now do research to answer the questions.


## Step 2: (Always) Review plan

KPop: Please review the plan in {{ plan_path }}.
- Are there errors in the plan?
- Is the plan self-consistent?
- Is the plan implementable?

Use up to 3 subagents if you think that could speed things up without sacrificing quality.


## Step 3: (Always) Revise plan

When you're done with the review, revise the plan to incorporate the review feedback. If there are answered questions at the bottom, incorporate them into the main plan, too.

If there are *really* still lingering questions, leave them at the bottom of {{ plan_path }} as multiple-choice questions.  Otherwise append to the bottom of {{ plan_path }}
```
---
[malvin] Ready for implementation.
```

Don't change code.