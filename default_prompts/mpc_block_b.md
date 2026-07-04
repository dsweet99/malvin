

2. If Step 1 could not falsify satisfaction of the user request (i.e., the user request is fully satisfied) write "DONE" to `{{ mpc_plan_path }}` and stop. Otherwise, continue.

3. KPop: Write a plan to satisfy the user request:
  - Write to `{{ mpc_plan_path }}`
  - If the plan is large or complex enough to require more than one phase, write a two-phase plan:
    (i) Work Phase: One detailed first phase.
    (ii) Deferred Phase: One rough summary of the remainder of the work.
  - Otherwise, if the plan can be completed in one phase (preferred), just write one phase and label it 'Work Phase'
