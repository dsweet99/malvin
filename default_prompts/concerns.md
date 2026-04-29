{{ coding_rules }}

---

Read `{{ review_path }}`, and address the reviewer's concerns. Be sure to also stay consistent with `{{ grounding_path }}` and `{{ plan_path }}` (if they exist and are not empty).

Never modify `{{ grounding_path }}`. If the reviewer's concerns require changing `{{ grounding_path }}`, write a line starting with `ABORT:` to `{{ result_path }}` explaining that `{{ grounding_path }}` must not change, instead of editing `{{ grounding_path }}`.

Use up to 3 parallel subagents.
