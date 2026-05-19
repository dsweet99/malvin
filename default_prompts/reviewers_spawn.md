{{ kpop }}

---

## Scope
Let's define the scope of this session as
- Directly related to the plan, `{{ plan_path }}`
- Preventing a quality gate from passing (any gate, any file, but only changes that relate to the failing gate)

## Review

KPop: Review in-scope code for these problems:
- Find inconsistencies with `{{ plan_path }}`.
- Find "cheats" to avoid violations of kiss metrics (run `kiss stats` to see a table of kiss metrics). Check, especially, import patterns and unit tests.
- Find serious bugs.
- Find serious redundancy or wasted work.
- Find poorly-tested code.
- Find very poorly-designed (bad SOC, leaky abstraction, overly-coupled, etc.) bits of code.
- Find non-idiomatic or inelegant code.

Be thorough (but in-scope). Be especially critical of cheating of any kind.

Write all discovered problems to {{ review_prep_path }} as the *Review List*.

Each item in the *Review List* should have the format
```md
- 1-2 sentence summary of problem. Relevant file paths & line numbers
```

