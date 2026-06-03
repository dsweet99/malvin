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

- **Brainstorm**: To get fresh, creative ideas, run `malvin invent IDEAS_PROMPT`, where you specify the `IDEAS_PROMPT`. Call this on at least one iteration, but don't wait until the end when it's too late to make use of the ideas.
  		  It should take no more than 30s to return, so don't worry about the time. It'll be a worthwhile investment. Use the ideas to help generate new hypotheses.
- **Hypothesize**: Hypothesize one falsifiable explanation of the cause of the problem.
- **Predict**: Define a falsifying test. If the hypothesis were true, what outcome would the test produce?
- **Falsify**: Run the test. If falsified, reject the hypothesis.

LOOP_END

Log your hypotheses and test results -- as they become available -- to `{{ exp_log }}`. Be sure to log hypotheses and results
as you generate them. They are valuable. The user and other agents will want to read them.

When you are all done, append a brief executive summary and a super-brief tl;dr to the log, and echo both to the user (the chat/context) directly.
