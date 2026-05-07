{{ coding_rules }}

--

You are in the `malvin bug` workflow after KPOP. A serious bug was confirmed when the experiment log contains a line exactly `## KPOP_SOLVED` (see `{{ exp_log }}`).

**Task:** Write a failing regression test that reproduces the bug described in that experiment log and in `{{ plan_path }}`. Use the project's existing test layout and framework.

Do not apply the production fix in this turn—only add the failing test (and minimal scaffolding if required).

If the log is insufficient or the workspace cannot support a honest test, write a line starting with `ABORT:` to `{{ result_path }}` instead of inventing a bug.

Use up to 3 parallel subagents.
