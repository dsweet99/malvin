Satisfy all constraints.


Scope Constraints:
{{ scope_constraints }}

General Constraints:
- All quality gates (see below) must pass.
- Agents can overfit tests to kiss's coverage estimator. Look for signs of this. Replace the overfitted tests with good unit tests
  - Make tests F.I.R.S.T.:  Fast, Independent, Repeatable, Self-Validating, Timely.
  - "Fast" means, ideally, under 1s. 2-4s would be ok for a very important test, but you should prefer to break tests up into smaller, more focused tests that run in less time.
  - For any non-trivial feature addition or for a bug fix, write at least one failing test first.
  - Call a function or method directly. Test a meaningful aspect of its behavior.
- Agents can overfit code to tests or evaluation metrics. Look for signs of this (e.g., special-casing). Excise the overfitted code.
  - Consider metamorphic tests and fuzzing tests to help prevent code from overfitting to tests.
- NO serious bugs in scope
- No serious time-complexity inefficiencies in scope
- No serious memory-complexity inefficiencies in scope
- Each unit test tests something meaningful. Simple tests are fine. Bogus tests are not.
- Any code you write should be idiomatic. (For example: Don't use ".inc" files in Rust.)
- Assert liberally. Good contracts make code reliable and maintainable.

- At times a task may seem formidable, but you are tenacious and your spirit is indominable. When faced with a large task, use this strategy
  - Find one piece of the task that you can handle. Define it clearly. Then completely focus on it until it is done.
  - Repeat. Remember, the journey of a thousand miles begins with a single step.

If you write new code:
- Stay in scope.


Quality Gates:

{{ quality_gates }}


Latest & up-to-date quality gate run output is in: `{{ quality_gates_path }}`.
