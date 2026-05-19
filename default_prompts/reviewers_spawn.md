{{ kpop }}

---

Pay attention to
- code that appears to implement `{{ plan_path }}`
- files unadded to git
- changes uncommitted to git
- files changed in recent (by count or datetime) commits


KPop: Review the codebase for these problems:
- Find inconsistencies with `{{ plan_path }}`.
- Find "cheats" to avoid violations of kiss metrics (run `kiss stats` to see a table of kiss metrics). Check, especially, import patterns and unit tests.
- Find serious bugs.
- Find serious redundancy or wasted work.
- Find poorly-tested code.
- Find very poorly-designed (bad SOC, leaky abstraction, overly-coupled, etc.) bits of code.
- Find non-idiomatic or inelegant code.

Be thorough. Be especially critical of cheating of any kind.

Write all discovered problems to {{ review_prep_path }} as a list.

Each item should have the format
```md
- [<SEVERITY_RATING>] 1-2 sentence summary of problem. Relevant file paths & line numbers
```

where SEVERITY_RATING is 1 (least severe), 2, 3, 4, 5 (most severe).
