
Read {{ review_prep_path }}.

Eliminate items in the *Review List* that are out of scope.

If no items remain, write *only* and *exactly* LGTM in {{ review_path }} and stop.

Otherwise:

- For each *bug* (not just *any* problem, bugs only) remaining in the *Review List*, write a failing regression test that exposes it.

Write your final review -- remaining problems only, based on the revised *Review List* -- to {{ review_path }}.
