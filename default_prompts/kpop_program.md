Satisfy all constraints.


Scope Constraints:
{{ scope_constraints }}

General Constraints:
- All quality gates (see below) to pass
- No serious bugs in scope
- No serious time-complexity inefficiencies in scope
- No serious memory-complexity inefficiencies in scope
- Each unit test tests something meaningful. Simple tests are fine. Bogus tests are not.
- Any code you write should be idiomatic. (For example: Don't use ".inc" files in Rust.)

If you write code:
- Stay in scope.
- Write real unit tests, even if it seems like you have to write a lot of them. Do your best. Don't use tricks or make superficial unit tests just to pass coverage gates.
- Look for signs of overfitting (e.g., special-casing for unit tests, not capturing the concept a test is meant to test, etc.)
- Write metamorphic tests to help prevent overfitting.
- Write fuzzing tests to help prevent overfitting. It's ok (even beneficial) to use a random seed, as long as you print it out (to aid debugging) *and* the true requirement is that the test pass for all seeds.

Quality Gates:

{{ quality_gates }}


Latest & up-to-date quality gate run output is in: `{{ quality_gates_path }}`.
