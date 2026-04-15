{{ coding_rules }}

---

Read {{ review_path }}, and address the reviewer's concerns.

Never modify grounding.md. If the reviewer's concerns require changing grounding.md, write a line starting with `ABORT:` to {{ result_path }} explaining that grounding.md must not change, instead of editing grounding.md.

Use up to 3 parallel subagents.
