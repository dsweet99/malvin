1. Write a plan to satisfy the user's request (see below):
  - Write to `{{ mpc_plan_path }}`
  - If you believe there is nothing left to do, write "DONE" to `{{ mpc_plan_path }}` and stop.
  - If the plan is large or complex enough to require more than one phase, write a two-phase plan:
    (i) Work Phase: One detailed first phase.
    (ii) Deferred Phase: One rough summary of the remaineder of the work.
  - Otherwise, if the plan can be completed in one phase (preferred), label the entire plane the 'Work Phase'
  - Include a plan to unit test. Think of all behaviors a reasonable individual would expect, and test for those. Think of all the ways an adversary could try to break or simply find mistakes in your code, and test for those.
  
2. KPOP: Check the plan for contradictions, errors, scope violation/creep, or plan inefficiency.
  - Append any questions you might have. Do research to answer your questions.
  - Leave no decisions open.
  
3. Revise the plan to address your review in step 2.

4. KPOP: Execute the work phase.

## User request (read this file):

`{{ user_request_path }}`