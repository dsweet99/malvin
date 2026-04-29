{{ coding_rules }}

Implement the plan in {{ plan_path }}, keeping it consistent with {{ grounding_path }}.

If you discover an unavoidable contradiction with `{{ grounding_path }}`, write a line starting with `ABORT:` to `{{ result_path }}` instead of changing `{{ grounding_path }}`.

Use up to 3 parallel subagents.