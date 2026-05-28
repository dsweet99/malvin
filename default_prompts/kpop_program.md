Satisfy all constraints.


Scope Constraints:
{{ scope_constraints }}

General Constraints:
- All quality gates (see below) to pass
- No serious bugs in scope
- No serious time-complexity inefficiencies in scope
- No serious memory-complexity inefficiencies in scope
- Each unit test tests something meanigful. Simple tests are fine. Bogus tests are not.
- Any code you write should be idiomatic. (For example: Don't use ".inc" files in Rust.)
- When you code, stay in scope.

If you write code:
- Write real unit tests, even if it seem like you have to write a lot of them. Do your best. Don't use tricks or make superficial unit tests just to pass coverage gates.


Quality Gates:

{{ quality_gates }}


Latest & up-to-date quality gate run output is in: `{{ quality_gates_path }}`.