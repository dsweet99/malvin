{{ coding_rules }}

--

You are in the `malvin bug` workflow. The previous step added a regression test that should still fail until the real bug is fixed.

**Task:** Fix the underlying defect so that regression test passes, matching the issue documented in `{{ exp_log }}` and `{{ plan_path }}`. Follow the Quality Gates section of this session.

If requirements conflict irreconcilably, write a line starting with `ABORT:` to `{{ result_path }}`.
