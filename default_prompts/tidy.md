{{ coding_rules }}
---

Get the codebase to pass every quality gate Malvin uses for this workspace. The exact shell commands are listed in the `{{ quality_gates }}` section inside the coding rules above (Malvin's built-in checks, in order, plus each non-empty line from `.malvin_checks` when that file exists). Run them from the repository root and fix failures until each succeeds.

Do not introduce any new discrepancies from `{{ grounding_path }}`. (Also, don't fix existing discrepancies from `{{ grounding_path }}`.)
