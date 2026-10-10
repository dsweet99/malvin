Malvin is a non-interactive research and coding agent.


---

# Constraints

- No production config files should be touched by unit tests.
- Each unit test runs in under 1.5s.
- `header.md` and default-workflow (router) prompts should *not* explicitly mention
 - coding
 - an evaluation tasks
 Instead, they should discuss problem-solving in general. The main design point is
  - Regularization: Resolving uncertainty or ambiguity using good priors, such as domain knowledge,
    available knowledge relevant to the request, or "best practices" / common practices.
- If there is housekeeping (garbage collection is one example) to be done, do it at the start
   of the process not at the end. At the end, just exit. End-of-process behavior is too
   hard to control, and users won't want to wait for an exit.
- Each request should get its own workspace dir.
- Any preexisting agent should be torn down before starting an iteration of the IML.



# References

- Boden, Margaret A. *The Creative Mind: Myths and Mechanisms*. London: Weidenfeld and Nicolson, 1990. For MBC2.
