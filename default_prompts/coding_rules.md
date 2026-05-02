
Use parallelized subagents (at most 4).

Work until the end without asking for user input. If you are uncertain about an implementation
detail, use your best judgement. There will always be an opportunity to revise later on.

`{{ quality_gates }}` lists the exact shell commands Malvin runs for this workspace (built-in gates plus optional `.malvin_checks` lines). Use that list as the source of truth for which languages and checks apply. Do NOT add a language that is not already present in the project.

Be sure that all applicable checks pass.

{{ quality_gates }}

- Run `kiss rules` before getting started so that you can avoid `kiss` VIOLATIONs.
- Run checks & tests frequently to avoid a big cleanup at the end.
  - You can save time by running the subset of tests reported by `kiss show-tests [FILE_OF_INTEREST [FILE_OF_INTEREST [...]]]]` while iterating.
- Do not write "documentation parity guards". Do not write comments. NEVER EDIT `{{ grounding_path }}`
  outside the **`malvin ground`** workflow (`write_grounding.md` may create `{{ grounding_path }}` when it is missing and only that file may be written; `improve_grounding.md` may edit `{{ grounding_path }}` alone when malvin invokes it).
- Keep each unit test's running time under 10 seconds (<1s would be great).
- Write code to fail fast. Assert liberally. DRY.
- Don't name files ".inc".  .rs and .py are the correct extensions.
- Keep code consistent with `{{ grounding_path }}` and `{{ plan_path }}` (if applicable).
- Use (up to 4) parallel subagents whereven possible.

## Nota Bene
ALL checks and tests should pass on ALL	files (not just the ones you modified). Don't tell me
 about "pre-existing" problems. We're here to work. To fix. Be tenacious. There's no excuse
 for not getting ALL checks and tests to pass on ALL files.
Don't touch .kissconfig ever.
Don't add `# noqa` except to ensure correct functioning of the code.
Don't cheat the tests. Make earnest attempts to pass the linters and unit tests in the spirit
 in which they were designed.
Your task is to get ALL checks and tests to pass on ALL files.
Do NOT create Rust code in a Python-only project, or Python code in a Rust-only project.
