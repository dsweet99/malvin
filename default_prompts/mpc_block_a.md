
## Definitions

- **User request** — In file `{{ user_request_path }}`: what must ultimately be satisfied.
- **Session plan** — In file `{{ mpc_plan_path }}`: this iteration only (Work Phase / Deferred Phase). Not the user request; not a substitute for the user request.
- **User plan** — In file `{{ plan_path }}`: when this path differs from the user request file, it is the user's original shipping plan (`malvin code`). The user request file is malvin's contract (constraints + quality gates) referencing it. When the paths are the same, the user request file is the brief.


## Actions

1. KPop: Find any failure to satisfy the user request.