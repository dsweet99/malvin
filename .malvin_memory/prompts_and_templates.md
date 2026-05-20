# Embedded prompts and `{{ key }}` templates

TRIGGER: review_plan, plan_path, placeholder, {{
ADVICE: Malvin `render_template` (`src/prompts/template.rs`) only replaces `{{ key }}` with spaces around the key. `{{key}}` is not expanded and leaves `{{` in the composed prompt. Scan with `malformed_brace_placeholders()`; fix spacing before debugging plan-path logic.
CONFIDENCE: 0

TRIGGER: prompt still contains, enforce_no_unresolved_braces, before ACP
ADVICE: Error `prompt still contains "{{" before ACP` means a prompt file still has unresolved `{{` after render—usually a mistyped placeholder in `default_prompts/*.md`, not `plan_resolve.inc`. Check `default_prompts/review_plan.md` first for `malvin plan` failures.
CONFIDENCE: 0

TRIGGER: compose_plan_prompt, malvin plan, plan_prompt
ADVICE: `malvin plan` builds the coder prompt via `compose_plan_prompt` in `src/cli/plan_flow/plan_prompt.rs` (header + coding rules + `review_plan.md`). Context from `plan_prompt_context` → `workflow_context` plus `plan_path`. Integration tests: `tests/plan_at_notation.rs`.
CONFIDENCE: 0

TRIGGER: embedded prompts, DEFAULT_PROMPTS, render test
ADVICE: Substring tests (`contains("{{ kpop }}")`) miss bad placeholders. Prefer `embedded_default_prompts_use_spaced_brace_placeholders` in `src/prompts/defaults.rs` plus a render test (`compose_plan_prompt_renders_embedded_review_plan_without_braces` or `store.render("review_plan.md", …)`).
CONFIDENCE: 0

TRIGGER: template.rs, include prompt, malformed_brace
ADVICE: `src/prompts/mod.rs` uses `include!("template.rs")`, so helpers are `crate::prompts::malformed_brace_placeholders`—not `crate::prompts::template::…`. Re-export from `lib.rs` if integration tests need it.
CONFIDENCE: 0
