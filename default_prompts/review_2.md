{{ coding_rules }}
---
{{ kpop }}

Run the same evaluator-equivalent acceptance checks as Review 1 and report any mismatch as blocking:

1. Create temporary files:
   - `jobs_ok.json` with:
     - `[{"id":"ingest","duration_ms":4,"deps":[]},{"id":"render","duration_ms":2,"deps":["ingest"]},{"id":"notify","duration_ms":1,"deps":["ingest"]},{"id":"archive","duration_ms":1,"deps":["render","notify"]}]`
   - `jobs_cycle.json` with:
     - `[{"id":"a","duration_ms":1,"deps":["c"]},{"id":"b","duration_ms":1,"deps":["a"]},{"id":"c","duration_ms":1,"deps":["b"]}]`
   - `jobs_bad_dep.json` with:
     - `[{"id":"a","duration_ms":3,"deps":["missing"]}]`
2. Verify `cargo run --quiet --release -- schedule --workers 2 jobs_ok.json` outputs exactly:
   - `[{"job":"ingest","worker":0,"start_ms":0,"end_ms":4},{"job":"notify","worker":1,"start_ms":4,"end_ms":5},{"job":"render","worker":0,"start_ms":4,"end_ms":6},{"job":"archive","worker":0,"start_ms":6,"end_ms":7}]`.
3. Verify failure cases return non-zero with one-line `ERR:`-prefixed stderr.

If behavior does not match, classify as blocking and require a concrete fix before approval.
---
KPop: Please review the codebase. Pay special attention to code that differs from branch main/.
- Is the code consistent with grounding.md (if it exists and is not empty) and `{{ plan_path }}` (if it exists and is not empty)?
- Is the code well-factored?
- Is the code idiomatic?
- Is the code elegant?

Write your review (problems only) to {{ review_path }}.

If everything is ok, just write LGTM in {{ review_path }}.
