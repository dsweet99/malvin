# Definition: KPop

[
KPop is short for "Karl Popper".
KPop may be referenced later on like a command, "KPop: <PROBLEM_STATEMENT_OR_QUESTION>"
]

[
Weak hypothesis (Michael Bennet): A weak hypothesis is a hypothesis that explains the observations while making as few additional
 commitments as possible. It remains consistent with as many possible unseen cases or completions as possible.
]

Apply this method to the PROBLEM_STATEMENT_OR_QUESTION.

Repeat until you think you've solved the PROBLEM_STATEMENT_OR_QUESTION:
LOOP_START

- **Hypothesize**: Hypothesize one falsifiable explanation of the cause of the problem. Prefer weak hypotheses.
- **Predict**: Define a falsifying test. If the hypothesis were true, what outcome would the test produce?
- **Falsify**: Run the test. If falsified, reject the hypothesis.

LOOP_END

Loop until you think you're done or you're up to max_hypotheses = `{{ max_hypotheses }}` iterations of the loop. You DO NOT need to use all allowed iterations if you are certain you've already solved the problem or answered the question.

Log your hypotheses and test results -- as they become available -- to `{{ exp_log }}`. Be sure to log hypotheses and results
as you generate them. They are valuable. The user and other agents will want to read them.

When you are all done, append a brief executive summary and a super-brief tl;dr to the log, and echo both to the user (the chat/context) directly.
