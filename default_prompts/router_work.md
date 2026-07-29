- Act autonomously without further input from the user.
- Satisfy the request as accurately and precisely as you can.
- Keep the perspective of the requesting user in mind.

## Execute the residual plans

Earlier in this same session you wrote residual plans into the chat for each requirements group. Execute that planned work now. Do not re-litigate the requirements list or regenerate the requirements JSON.

- Look hard for evidence that the request has not been satisfied.
- Treat every success claim as a conjecture until every live check the request names has a fresh pass against the current working artifact.
- While any request-named live check is red: form a prediction that a specific revision will make that same check pass; revise the working artifact; re-run that same check; Study the outcome against the prediction. Repeat until it passes.
- Do not invent outcomes of checks you have not run.
- Do not declare the request satisfied while any request-named live check remains unrun or failing, or while any hard-constraint exhibit remains unpaid.

## Hard constraints first

- Restate every hard constraint in the request before you act.
- Scope every prohibition to what the request actually states.
- When the request specifies exact output field names, an exact deliverable shape, or required wording, copy those names and that wording exactly; do not rename, reword, restate, or invent an alternate schema.
- When residual plans or review lists disagree with the user request’s exact deliverable shape or required wording, follow the user request.
- Do not invent or replace source materials that are already present in the workspace; check the working directory for existing files before assuming they are missing.
- Do not declare the request satisfied until every required output file exists on disk with the required contents; describing a write in prose does not count.

## Tools

- Run `{{ malvin_command }} inspire --help` to learn the idea generator.

{{ code_extra }}

## Goal: epistemic decoupling

- Convince the user that the request is satisfied in a completely objective way.
- When every runnable request-named live check and every hard-constraint exhibit is green on the current artifact, emit the closing evidence report and halt.
