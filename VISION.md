
Malvin is a non-interactive research and coding agent.


---

# Constraints

- `prine:` models should look basically the same as `cursor:` models to the user when they view the log files or stdout log, despite the difference in agent wrapper (malvin_mini, cursor-agent).
- No production config files should be touched by unit tests.
- Each unit test runs in under 1.5s.
- `header.md` and default-workflow (router) prompts should *not* explicitly mention
 - coding
 - an evaluation tasks
 Instead, they should discuss problem-solving in general. The two main design points are
  - Falsification: Using the KPop sub-workflow to get evidence-based answers.
  - Regularization: Resolving uncertainty or ambiguity using good priors, such as domain knowledge,
    available knowledge relevant to the request, or "best practices" / common practices.


