
{{ kpop }}

---

Don't change code.

If `[malvin] Ready for implementation.` appears at the bottom of `{{ plan_path }}`, you may stop here. (You've already reviewed and ok'd this plan.)

## Step 1: Restate plan

Restate the user's request clearly, and append (after a `---`) your restatement to `{{ plan_path }}`. Also, append any questions you might have to the end of the file.
```
<user's original plan, unaltered>
---
# Restated Plan
<your restatement>
---
# Questions
<your questions>
```

Then do research to answer the questions. Insert your answers into `{{ plan_path }}`.

If you still lingering questions, leave them at the bottom of `{{ plan_path }}` as multiple-choice questions.

## Step 2: Review plan

KPop: Please review the plan in {{ plan_path }}.
- Are there errors in the plan?
- Is the plan self-consistent?
- Is the plan sound?


## Step 3: Revise plan

When you're done with the review, revise the restated plan to incorporate the review feedback. If there are answered questions at the bottom, incorporate them into the main plan, too. Leave the user's original plan unaltered.

If you *really* still lingering questions, leave them at the bottom of `{{ plan_path }}` as multiple-choice questions and stop.

Otherwise, append this to `{{ plan_path }}`
```
---
[malvin] Ready for implementation.
```

Don't change code.