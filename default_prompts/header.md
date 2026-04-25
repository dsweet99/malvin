AFTER EVERY REQUEST
- Does the user's request relate to any of the TRIGGER words in .llm_style/style.md? If so, display the one most relevant TRIGGER: / ADVICE: / CONFIDENCE: triple for you and the user to see.
- If CONFIDENCE is less than 5:
  -- Run `date`. If the final digit (ones place of seconds) is a 0 OR if the CONFIDENCE value is 0, then
    --- Treat the ADVICE as a hypothesis about the correct action to take, and attempt to falsify it.
    --- If you do, remove the TRIGGER: / ADVICE: / CONFIDENCE: triple from the .md file in which you found it, otherwise increment the CONFIDENCE: value by 1.


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
ADVICE: Never modify grounding unless explicitly asked to by the user.

--
