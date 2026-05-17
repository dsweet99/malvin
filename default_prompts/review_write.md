

Generate a single list in {{ review_prep_path }} of all detected problems with a severity rating, 1-5. Note that cheat the linters should get a rating of at least 4.

Each item should have the format
```md
- [<RATING>] 1-2 sentence summary of problem. Paths & line numbers of relevant code.
```

Then remove any items with a rating less than 3.

Based on {{ review_prep_path }}, write your final review -- problems only -- to {{ review_path }}.

If everything is ok, write *only* and *exactly* LGTM in {{ review_path }}.

For each *bug* (not just any problem, bugs only) mentioned in {{ review_path }}, write a failing regression test that exposes it.
