{{ coding_rules }}

Please review the codebase. Pay special attention to code that differs from branch main/.
- Is the code consistent with grounding.md and `{{ plan_path }}`?
- Are there bugs?
- Is there avoidable redundancy or wasted work?
- Is the code well-tested?

For every bug you find, write a failing test that exposes it.

Write your review (problems only) to {{ review_path }}.

If everything is ok, just write
```
LGTM
```
in {{ review_path }}.
