
Malvin is a non-interactive research and coding agent.


---

# Constraints

- `openrouter:` models should look basically the same as `cursor:` models to the user when they view the log files or stdout log, despite the difference in agent wrapper (malvin_mini, cursor-agent).
- No production config files should be touched by unit tests.
- DeepSWE is one of many tasks for which malvin is designed, thus malvin should not have specific support for solving DeepSWE tasks. The repo just provides a Python ops/ tool to run DeepSWE evals of malvin on Modal. We
   *should* support good setup of the sandboxes (e.g., networking, dependencies).
- Each unit test runs in under 1.5s.
- `header.md` and default-workflow (router) prompts should *not* explicitly mention
 - coding
 - an evaluation tasks
 Instead, they should discuss problem-solving in general. The two main design points are
  - Falsification: Using the KPop sub-workflow to get evidence-based answers.
  - Regularization: Resolving uncertainty or ambiguity using good priors, such as domain knowledge,
    available knowledge relevant to the request, or "best practices" / common practices.


# malvin-mini
