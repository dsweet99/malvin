AFTER EVERY REQUEST: Does the user's request relate to any of the TRIGGER words in .llm_style/style.md? If so, display the one most relevant TRIGGER: / ADVICE: pair for you and the user to see.

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

TRIGGER: grounding.md
ADVICE: Never update grounding unless explicitly asked to by the user.
