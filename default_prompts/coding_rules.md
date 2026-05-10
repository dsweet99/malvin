
# Repo Coding Rules

Use parallelized subagents (at most 4).

Work until the end without asking for user input. If you are uncertain about an implementation
detail, use your best judgement. There will always be an opportunity to revise later on.

The Quality Gates section lists the exact shell commands from `.malvin_checks` (non-empty lines, in order). Malvin creates that file with layout-appropriate defaults when it is missing. Use that list as the source of truth for which languages and checks apply. Do NOT add a language that is not already present in the project.

## Quality Gates
Be sure that all applicable checks pass.

{{ quality_gates }}

When presented with failures or violations, respond to them earnestly. Improve the code in a way that respects the spirit of the quality-gate feedback.

## Tips and Soft Requirements
- Run `kiss rules` before getting started so that you can avoid `kiss` VIOLATIONs.
- Run checks & tests frequently to avoid a big cleanup at the end.
  - You can save time by running the subset of tests reported by `kiss show-tests [FILE_OF_INTEREST [FILE_OF_INTEREST [...]]]]` while iterating.
- Do not write "documentation parity guards". Do not write comments.
- Keep each unit test's running time under 10 seconds (<1s would be great).
- Write code to fail fast. Assert liberally. DRY.
- Don't name files ".inc".  .rs and .py are the correct extensions.
- Keep code consistent with `{{ plan_path }}` when this run includes a plan file.
- Use (up to 4) parallel subagents whereven possible.

## Nota Bene & Hard Requirements
ALL checks and tests should pass on ALL	files (not just the ones you modified). Don't tell me
 about "pre-existing" problems. We're here to work. To fix. Be tenacious. There's no excuse
 for not getting ALL checks and tests to pass on ALL files.
Don't touch .kissconfig ever.
- Don't touch .kissignore unless you think you've found a bug in `kiss check`.
Don't add `# noqa` except to ensure correct functioning of the code.
Don't cheat the tests. Make earnest attempts to pass the linters and unit tests in the spirit
 in which they were designed.
Your task is to get ALL checks and tests to pass on ALL files.
Do NOT create Rust code in a Python-only project, or Python code in a Rust-only project.
