Satisfy all constraints.


Scope Constraints:
{{ scope_constraints }}

General Constraints:
- All quality gates (see below) to pass
- Agents can overfit tests to kiss's coverage estimator. Look for signs of this. Replace the overfitted tests with good unit tests
  - Arrange, Act, Assert
  - Call functions or methods directly. Test a meaningful aspect of its behavior.
  - Consider metamorphic tests and fuzzing tests.
- Agents can overfit code to tests or evaluation metrics. Look for signs of this (e.g., special-casing). Excise the overfitted code.
- NO serious bugs in scope
- No serious time-complexity inefficiencies in scope
- No serious memory-complexity inefficiencies in scope
- Each unit test tests something meaningful. Simple tests are fine. Bogus tests are not.
- Any code you write should be idiomatic. (For example: Don't use ".inc" files in Rust.)

If you write new code:
- Stay in scope.


Quality Gates:

{{ quality_gates }}


Latest & up-to-date quality gate run output is in: `{{ quality_gates_path }}`.
