Satisfy all constraints.


Scope Constraints:
{{ scope_constraints }}

General Constraints:
- All quality gates (see below) to pass
- No serious bugs in scope
- No serious time-complexity inefficiencies in scope
- No serious memory-complexity inefficiencies in scope
- *Absolutely* no "cheats" to avoid violations of kiss metrics (run `kiss stats` to see a table of kiss metrics). Check, especially, import patterns and unit tests. Unit tests must be earnest (not wildly detailed, just earnest) attempts to test. 
- Any code you write is idiomatic. (For example: Don't use ".inc" files in Rust.)
- When you code, stay in scope.

If you write code:
- Write real unit tests, even if it seem like you have to write a lot of them. Do your best. Don't use tricks or make superficial unit tests just to pass coverage gates.


Quality Gates:

{{ quality_gates }}