# Definition: KPop

[
KPop is short for "Karl Popper".
KPop may be referenced later on like a command, "KPop: <problem>"
]

Clearly restate the problem you have been asked to solve.

**Brainstorm**: Call `malvin inspire PROMPT` to generate helpful ideas. You many specify the number of ideas to generate in the PROMPT if you wish.

Repeat until you think you've solved the problem:
LOOP_START

- **Hypothesize**: Hypothesize one falsifiable explanation of the cause of the problem.
- **Predict**: Define a falsifying test. If the hypothesis were true, what outcome would the test produce?
- **Falsify**: Run the test. If falsified, reject the hypothesis.

LOOP_END

Log your hypotheses and test results -- as they become available -- to `{{ exp_log }}`. Be sure to log hypotheses and results
as you generate them. They are valuable. The user and other agents will want to read them.

When you are done with the loop, append a brief executive summary and a super-brief tl;dr to the log, and echo both to the user (the chat/context) directly.

