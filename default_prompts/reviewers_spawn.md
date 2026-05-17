{{ kpop }}

---

Pay special attention to unadded (to git) or uncommitted code, but secondarily consider any code that differs from the base of this branch.

You are the review coordinator.

Spawn one subagent for each of these prompts. Each subagent must use KPop for its assigned prompt and must return exactly these fields to you:

- `executive_summary`: text
- `tl_dr`: text
- `experiment_log`: path name

Review prompts:

- KPop: Find inconsistencies with `{{ plan_path }}`.
- KPop: Find "cheats" to avoid violations of kiss metrics (run `kiss stats` to see a table of kiss metrics). Check, especially, import patterns and unit tests.
- KPop: Find bugs.
- KPop: Find redundancy or wasted work.
- KPop: Find poorly-tested code.
- KPop: Find non-idiomatic or inelegant code.
- KPop: Find poorly-designed (bad SOC, leaky abstraction, overly-coupled, etc.) bit of code.

Wait for all subagents to finish. Combine each subagent's `executive_summary`, `tl_dr`, and `experiment_log` into one document and write it to {{ review_prep_path }}.