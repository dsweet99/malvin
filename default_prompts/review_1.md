{{ coding_rules }}
---
{{ kpop }}

MANDATORY: Prefer parallel investigation. If the Task tool is available, launch up to 3 parallel subagents to:
- Exploring different areas of the codebase simultaneously
- Testing multiple hypotheses in parallel
If the Task tool is unavailable, use parallel non-Task tools where possible; otherwise proceed sequentially and explicitly note the fallback and any resulting coverage limits.
Do NOT choose sequential execution when a parallel option is available.
---
KPop: Please review the codebase. Pay special attention to code that differs from branch main/.
- Is the code consistent with grounding.md (if it exists and is not empty) and `{{ plan_path }}` (if it exists and is not empty)?
- Are there bugs?
- Is there avoidable redundancy or wasted work?
- Is the code well-tested?

Run the evaluator-equivalent acceptance checks below and record failures as blocking items if output differs.

1. Create temporary files for each case:
   - `jobs_ok.json` with:
     - `[{"id":"ingest","duration_ms":4,"deps":[]},{"id":"render","duration_ms":2,"deps":["ingest"]},{"id":"notify","duration_ms":1,"deps":["ingest"]},{"id":"archive","duration_ms":1,"deps":["render","notify"]}]`
   - `jobs_cycle.json` with:
     - `[{"id":"a","duration_ms":1,"deps":["c"]},{"id":"b","duration_ms":1,"deps":["a"]},{"id":"c","duration_ms":1,"deps":["b"]}]`
   - `jobs_bad_dep.json` with:
     - `[{"id":"a","duration_ms":3,"deps":["missing"]}]`
2. Verify `cargo run --quiet --release -- schedule --workers 2 jobs_ok.json` equals:
   `[{"job":"ingest","worker":0,"start_ms":0,"end_ms":4},{"job":"notify","worker":1,"start_ms":4,"end_ms":5},{"job":"render","worker":0,"start_ms":4,"end_ms":6},{"job":"archive","worker":0,"start_ms":6,"end_ms":7}]`.
3. Verify failure cases return non-zero and exactly one-line `ERR:`-prefixed stderr.

For every bug you find, write a failing regression test that exposes it.

Write your review (problems only) to {{ review_path }}.

If everything is ok, just write LGTM in {{ review_path }}.
