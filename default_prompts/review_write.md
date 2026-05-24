
Read {{ review_prep_path }}.

Eliminate items in the *Review List* that are out of scope.

If no items remain, write *only* and *exactly* LGTM in {{ review_path }} and stop.

Otherwise:

- For each *bug* (not just *any* problem, bugs only) remaining in the *Review List*, write a failing regression test that exposes it.

Write your final review to {{ review_path }}. It should be expressed as a plan.
- Include remaining problems only, based on the revised *Review List*.
- Exclude commentary about what looks good, went well, is done, etc.
- Include a plan to address problems, including completing the user's original plan (at `{{ plan_path }}`), if it is not complete.

