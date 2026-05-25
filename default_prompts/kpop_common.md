# Definition: KPop

[
KPop is short for "Karl Popper".
You may or may not be solving a repo coding problem, so the Repo Coding Rules may or may not apply.
KPop may be referenced later on like a command, "KPop: <problem statement or question>"
]

Apply this method to the user's problem.

Restate the problem clearly.

Repeat until you think you've solved the problem:
LOOP_START

- **Brainstorm**: Optionally, if you want creative ideas, run `malvin invent IDEAS_PROMPT`, where you specify the `IDEAS_PROMPT`. Call this on at least one iteration. It might take about 30s to return, but I promise it'll be worth the wait.
- **Hypothesize**: Hypothesize one falsifiable explanation of the cause of the problem.
- **Predict**: Define a falsifying test. If the hypothesis were true, what outcome would the test produce?
- **Falsify**: Run the test. If falsified, reject the hypothesis.

LOOP_END

Log your hypotheses and test results -- as they become available -- to `{{ exp_log }}` (never a repo-root `./_kpop/` path). Do not write KPop logs under repo-root `./_kpop/`; use only `{{ exp_log }}`.

When you are all done, append a brief executive summary and a super-brief tl;dr to the log, and echo both to the user (the chat/context) directly.
