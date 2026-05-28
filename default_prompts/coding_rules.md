
# Repo Coding Rules

Work until the end without asking for user input. If you are uncertain about an implementation
detail, use your best judgement. There will always be an opportunity to revise later on.

The Quality Gates section lists the exact shell commands from `.malvin/checks` (non-empty lines, in order). Malvin creates that file with layout-appropriate defaults when it is missing. Use that list as the source of truth for which languages and checks apply. Do NOT add a language that is not already present in the project.

## Quality Gates

Be sure that all applicable checks pass.
Malvin writes quality-gate command output to `{{ quality_gates_log }}`; inspect that log when you need failure details.

{{ quality_gates }}

When presented with failures or violations, respond to them earnestly. Improve the code in a way that respects the spirit of the quality-gate feedback.

When you want to iterate quickly on a bit of code, try these fast tools:

- `cargo check` - Tells whether code *would* compile, but doesn't compile it.
- `kiss test <filename>` - Runs just the subset of Rust or Python tests that test a given filename. Heuristic but fast b/c you aren't running all tests.

`ruff check` and `kiss check` are also very fast, even when applied to the entire codebase.

## Tips and Soft Requirements

- Run `kiss rules` before getting started so that you can avoid `kiss` VIOLATIONs.
- Run `kiss --help` to familiarize yourself with kiss's coding tools, like `test` and `mv`. `kiss test` can be especially helpful in reducing the unit test burden in a coding loop.
- Do not write "documentation parity guards". Do not write comments.
- Keep each unit test's running time under 10 seconds (<1s would be great).
- Write code to fail fast. Assert liberally. DRY.
- Don't name source code files ".inc". .rs and .py are the correct extensions.
- Keep code consistent with `{{ plan_path }}` when this run includes a plan file.
- Use (up to 4) parallel subagents whereven possible.
- Do not write tests for tests. That's silly overkill.
- In tests, avoid doing things that take a long or highly-variable amount of time. Also avoid external services if at all possible. They can be unreliable, and the operators might get annoyed if we keep using them just for our tests.

## Nota Bene & Hard Requirements

ALL checks and tests should pass on ALL    files (not just the ones you modified). Don't tell me
 about "pre-existing" problems. We're here to work. To fix. Be tenacious and intrepid. There's no excuse
 for not getting ALL checks and tests to pass on ALL files.
Do not modify protected workspace files: `.kissconfig`, `.kissignore`, `.malvin/checks`, or `.malvin/config.toml`. Malvin snapshots them before agent work and restores them after each coder step, so edits to those files will not stick; fix violations in application code instead. Don't call `kiss clamp` or `kiss mimic`. You can't get out of it. You need to restructure the code to satisfy `kiss check`.
Once in a while, `kiss check` will show *many* violations. Don't get overwhelmed or intimidated by that. Keep your composure. Maybe use your TODO list. Maybe make a plan. Remember that a journey of a thousand miles begins with a single step.
Don't add `# noqa` except to ensure correct functioning of the code.
Don't cheat the tests. Make earnest attempts to pass the linters and unit tests in the spirit
 in which they were designed.
Your task is to get ALL checks and tests to pass on ALL files.
Do NOT create Rust code in a Python-only project, or Python code in a Rust-only project.
DO NOT add, commit, or otherwise perform a modifying operation on git. (You may inspect the log or diff to files, but no changing git.)
