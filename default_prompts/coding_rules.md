
AFTER EVERY REQUEST: Does the user's request relate to any of the TRIGGER words in style.md? If so, display the one most relevant TRIGGER: / ADVICE: pair for you and the user to see.

--

### Definitions: Claims vs Hypotheses

- Label uncertain reasoning as Hypothesis; only use Claim with explicit evidence.
- Claims must cite evidence (code refs, logs, metrics). Otherwise, downgrade to Hypothesis.
- For each Hypothesis, include:
  - Hypothesis: concise, falsifiable statement.
  - Predictions: measurable outcomes if true.
  - Test: minimal experiment (setup, variables, metrics, pass/fail).
  - Confounders: likely alternatives and controls.
- Language:
  - Hypothesis: “suggests”, “may”, “indicates”.
  - Claim (with evidence): “shows”, “demonstrates”, “causes”.
- Label any statement which is a hypothesis as such.

--


Use parallelized subagents (at most 4).

Work until the end without asking for user input. If you are uncertain about an implementation
detail, use your best judgement. There will always be an opportunity to revise later on.

Be sure that all checks pass:
- ruff check
- cargo clippy --all-targets --all-features -- \
    -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::cargo \
    -A clippy::must_use_candidate \
    -A clippy::missing_errors_doc \
    -A clippy::missing_panics_doc
- kiss check
and all unit tests pass:
- cargo test


Run checks & tests frequently to avoid a big cleanup at the end.

## Nota Bene
ALL checks and tests should pass on ALL	files (not just the ones you modified). Don't tell me
 about "pre-existing" problems. We're here to work. To fix. Be tenacious. There's no excuse
 for not getting ALL checks and tests to pass on ALL files.
Don't touch .kissconfig ever.
Don't add `# noqa` except to ensure correct functioning of the code.
Don't cheat the tests. Make earnest attempts to pass the linters and unit tests in the spirit
 in which they were designed.
Your task is to get ALL checks and tests to pass on ALL files.
