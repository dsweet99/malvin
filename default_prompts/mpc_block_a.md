
## Definitions

- **User request** — In file `{{ user_request_path }}`: what must ultimately be satisfied.
- **Session plan** — In file `{{ mpc_plan_path }}`: this iteration only (Work Phase / Deferred Phase). Not the user request; not a substitute for the user request.
- **User plan** — In file `{{ plan_path }}`: when this path differs from the user request file, it is the user's original shipping plan (`malvin code`). The user request file is malvin's contract (constraints + quality gates) referencing it. When the paths are the same, the user request file is the brief.


## Actions

1. KPop: Find (i) failures to satisfy the user request to the letter, (ii) failures to satisfy the user request in spirit, (iii) failures to generalize, or simply, (iv) failures to correctly interpret the user request. Report on the failures that you find.

   Before testing, read the user request and state:
   - The **contracts** this work must honor (e.g., "any command that accepts a format flag must produce valid output in that format regardless of other flags").
   - The **invariants** a consumer would rely on (e.g., "every subcommand that doesn't need input files must work without them").
   - The **ambiguous phrases** that have more than one plausible reading, and which reading a first-time reader would most naturally adopt.

   When your work introduces a new path alongside an existing one that handles the same job, apply **precedent parity**: study how the established path behaves in every circumstance — including when inputs are wrong, incomplete, or incompatible — and confirm the new path preserves that behavior unless the user request explicitly changes it. A new path that works on typical inputs but mishandles cases the old path already handles correctly is a failure, even if the request never names those cases.

   Then verify against those contracts and invariants — not against the implementation's own assumptions. A requirement is only satisfied when an adversarial reader of the user request, seeing only the user request and the output, would agree it is met. Do not validate an interpretation by confirming it against itself.

   Passing checks you have already run is necessary but not sufficient. Ask what an independent reviewer would probe that you have not: inputs that violate unstated prerequisites, combinations you did not try, and failure modes the request implies but never names. Verify those before concluding the work is complete.

   For each feature, test it in combination with every other feature it could interact with. A feature that works in isolation but fails when composed with another is a failure.
