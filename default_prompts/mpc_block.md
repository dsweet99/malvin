1. Write a plan to satisfy the user's request (see below):
  - Write to `{{ mpc_plan_path }}`
  - If you believe that all work is complete, i.e., that the user's request is satisfied, write "DONE" to `{{ mpc_plan_path }}` and stop.
  - If the plan is large or complex enough to require more than one phase, write a two-phase plan:
    (i) Work Phase: One detailed first phase.
    (ii) Deferred Phase: One rough summary of the remainder of the work.
  - Otherwise, if the plan can be completed in one phase (preferred), label the entire plan the 'Work Phase'
  - Look for bugs, inefficiencies, and overfitting in any code already implemented to satisfy some portion of the user's request. Include plans to fix.
  
2. Apply KPOP to this: Check the plan for contradictions, errors, scope violation/creep, or plan inefficiency.
  - Append any questions you might have. Do research to answer your questions.
  - Leave no decisions open.
  
3. Revise the plan to address your review in step 2.

4. Apply KPOP to this: Execute the work phase.

## User request (read this file):

`{{ user_request_path }}`