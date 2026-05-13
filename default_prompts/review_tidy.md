{{ coding_rules }}
---
{{ kpop }}

Prefer parallel investigation. Launch up to 3 parallel subagents to:
- Explore different areas of the codebase simultaneously
- Test multiple hypotheses in parallel
Do NOT choose sequential execution when a parallel option is available.

<!-- malvin:review_tidy_turn_v1 -->

---

KPop: Please review uncommitted code, even unadded files.
- Is the code consistent with `{{ plan_path }}` when a plan file exists for this run, and with the Quality Gates section?
- Is there evidence of cheating to avoid violations of kiss metrics (run `kiss stats` to see a table of kiss metrics)?
- Are new unit tests legitimate unit tests? Are there any that cheat to game kiss's code-coverage rules?
- Were any new bugs introduced?
- Is there avoidable redundancy or wasted work?

For every bug you find, write a failing regression test that exposes it.

Write your review (problems only) to {{ review_path }}.

If everything is ok, write *only* and *exactly* LGTM in {{ review_path }}. 
