
Read the files in {{ reviewers_subdir }}, then
1. Rate all of the findings for seriousness on a scale of 1-5. Make up your own mind about the level of seriousness.
2. Discard anything rated 1.

Write your review (problems only) to {{ review_path }}.

If everything is ok, write *only* and *exactly* LGTM in {{ review_path }}.

For each remaining bug finding after the seriousness filtering above, write a failing regression test that exposes it before writing the final review.
