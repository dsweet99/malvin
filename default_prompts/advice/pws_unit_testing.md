description: Problems Worth Solving for unit testing code


1. **Missing behavioral contract**
   Important behavior is implemented but not expressed by tests. Future changes can silently alter user-visible rules because the intended outcome is not recorded in an executable form.

2. **Weak oracle**
   A test exercises code but makes assertions too broad, too shallow, or unrelated to the behavior at risk. It can pass even when the important result is wrong.

3. **Brittle implementation coupling**
   Tests depend on private structure, incidental call order, exact query shape, or other implementation details that are not part of the contract. Safe refactoring therefore causes widespread test failures.

4. **Uncontrolled nondeterminism**
   Correctness depends on time, randomness, scheduling, network responses, shared state, or execution order that the test does not control. Failures become intermittent and difficult to reproduce.

5. **Unrepresented boundary**
   Behavior changes at an empty value, limit, transition, failure response, or other boundary, but tests cover only ordinary examples. The cases most likely to contain defects remain unspecified.

6. **Mocked-away behavior**
   Mocks replace so much real behavior that tests primarily confirm their own setup. Components appear correct in isolation while their actual composition remains unverified.

7. **Scattered test setup**
   One conceptual scenario requires repetitive or coordinated fixture construction across many tests. Changes to the domain model produce noisy edits unrelated to the behavior under test.

8. **Opaque failure**
   A failing test does not identify the broken rule because its name, assertions, fixture data, or diagnostics obscure the relevant difference. Investigation requires reconstructing the test rather than reading the failure.

9. **Test-production drift**
   Tests rely on substitutes, schemas, configuration, or assumptions that no longer match production. The suite passes against a system that is meaningfully different from the one users run.

10. **Misallocated coverage**
    Tests concentrate on code that is easy to exercise while leaving high-risk rules, destructive operations, compatibility boundaries, and complex state transitions comparatively untested. Test effort does not reflect the cost or likelihood of failure.
