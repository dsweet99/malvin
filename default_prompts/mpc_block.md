
## Definitions

- **User request** — In file `{{ user_request_path }}`: what must ultimately be satisfied.
- **Session plan** — In file `{{ mpc_plan_path }}`: this iteration only (Work Phase / Deferred Phase). Not the user request; not a substitute for the user request.
- **User plan** — In file `{{ plan_path }}`: when this path differs from the user request file, it is the user's original shipping plan (`malvin code`). The user request file is malvin's contract (constraints + quality gates) referencing it. When the paths are the same, the user request file is the brief.


## Actions

1. Write a plan to satisfy the user request:
  - Write to `{{ mpc_plan_path }}`
  - If you believe that all work is complete, i.e., that the user request is satisfied, write "DONE" to `{{ mpc_plan_path }}` and stop, i.e. skip the rest of the following steps.
  - If the plan is large or complex enough to require more than one phase, write a two-phase plan:
    (i) Work Phase: One detailed first phase.
    (ii) Deferred Phase: One rough summary of the remainder of the work.
  - Otherwise, if the plan can be completed in one phase (preferred), label the entire plan the 'Work Phase'
  - Look for bugs, inefficiencies, and overfitting in any code already implemented to satisfy some portion of the user request. Include plans to fix.
  
2. KPop: Falsify the plan, looking for contradictions, errors, scope violation (w.r.t. the user request), or plan inefficiency.
  - Append any questions you might have. Do research to answer your questions. Make the best decisions you can given the information you have.
  
3. Revise the plan to address your review in step 2.

4. KPop: Execute the work phase.
