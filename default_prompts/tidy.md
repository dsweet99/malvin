{{ coding_rules }}
---

Get the codebase to pass every quality gate Malvin uses for this workspace. The exact shell commands are listed in the Quality Gates section inside the coding rules above. Run them from the repository root and fix failures until each succeeds.

Do not introduce any new discrepancies from `{{ grounding_path }}`. (Also, don't fix existing discrepancies from `{{ grounding_path }}`.)
