

Read {{ review_prep_path }}.
Ignore any item with a rating of 1.

For each *bug* (not just *any* problem, bugs only) mentioned in {{ review_prep_path }}, with a rating > 1, write a failing regression test that exposes it.

Write your final review -- problems only -- to {{ review_path }}.

If there are no problems (rating > 1), write *only* and *exactly* LGTM in {{ review_path }}.
