# MPC Request

- USER_REQUEST = current contents of `{{ user_request_path }}`
- Append to `{{ user_request_path }}`. Preserve the original USER_REQUEST at the top.
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
  - *Don't* change anything unnecessary ("Current State" should help with this, especially).
  - Don't mark anything "optional".
  - Don't leave any decisions open.
  - Don't change code. Plan only.
  - Append a single exact line `## MPC_DONE` to `{{ user_request_path }}` when **planning has converged**: your re-audit of the working tree and latest experiment log shows the current Work Phase is already satisfied, and you would not change Current State, Q&A, or Phases on another pass. Otherwise do **not** append `## MPC_DONE`. Implementer close-out (gates, `## KPOP_SOLVED`) does **not** block `## MPC_DONE` once that close-out is complete — it means you have no further plan to add.

KPop: the MPC Request above
