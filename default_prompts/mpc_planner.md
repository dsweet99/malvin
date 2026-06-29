# MPC Request

- USER_REQUEST = current contents of `{{ user_request_path }}`
- Append to `{{ user_request_path }}`. Preserve the original  USER_REQUEST at the top.
  - *If there is NO more work to be done, i.e. if the USER_REQUEST is already satisfied*: Append a single exact line `## MPC_DONE` to `{{ user_request_path }}`.
  - *If there is more work to be done*:
    - Write section "Current State": describe codebase state adjacent to (affects or might be affected by) USER_REQUEST. Include structure and behaviors.
      - Scope: The changes needed to satisfy USER_REQUEST; Current State is the baseline only.
    - Write section "Q&A":
      - Append any questions you have.
      - Research the codebase (or web or write small scripts) to answer them.
    - Write section "Phases":
      - Definition of *phase*: (i) a set of code changes, and (ii) meaningful validation of them.
      - Rewrite the user's request as one or two subsections:
        - "Work Phase": as much work as can reasonably fit in one phase; include concrete validation steps.
        - "Deferred Phase": short summary of everything else. Omit if all work fits in Work Phase.
      - The first phase is detailed; the second is deliberately not.
  - *Do* satisfiy the entire user request.
  - *Don't* change anything unnecessarily ("Current State" should help with this, especially).
  - Don't mark anything "optional".
  - Don't leave any decisions open.
  - Don't change code. Plan only.


KPop: the MPC Request above
