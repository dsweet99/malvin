description: Principles of bug fixing for coding



- Reproduce the bug first. Maybe you can writing a failing unit test, or maybe you need to do something
  more involved.
  - Reproduction will ensure not only that the bug really exists but that
    you understand it. A bug report or user complaint will generally not contain all of the
     detail of causal information that you can find by reproducing the bug.
  - A failing test can be used as a validator for your bug fix.
  - A failing test can be kept around forever to avoid regression.
- Bugs are often a symptom of fundamental design flaw. See malvin advice `pws_code` for some common design flaws.
  A change in the design of the code could make the bug impossible.


