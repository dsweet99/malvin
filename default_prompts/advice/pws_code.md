description: Problems Worth Solving for coding


1. **Diffuse concept**
   A coherent concept is used wholly or partially in multiple places but has no explicit representation. It should probably be represented by a named type, module, interface, function, or other abstraction.

2. **Hidden invariant**
   Correctness depends on a property remaining true, but that requirement is implicit rather than clearly expressed or enforced by the code.

3. **Representable invalid state**
   The code's types or data structures permit semantically invalid states, combinations, or configurations. Change the representation so that invalid states are impossible or materially harder to construct.

4. **Scattered change**
   A single conceptual change predictably requires coordinated edits in multiple otherwise separate locations. The structure fails to localize that kind of change.

5. **Mixed responsibility**
   A module, type, or function contains multiple concepts that evolve for substantially different reasons. Unrelated changes therefore interact with the same abstraction.

6. **Misplaced responsibility**
   Behavior resides somewhere that does not own the concepts or data it primarily reasons about, forcing one part of the system to understand another part's internals.

7. **Duplicated policy**
   The same rule, decision, transformation, or algorithmic choice is independently expressed in multiple places, even if the implementations are superficially different.

8. **Implicit protocol**
   Correctness depends on operations occurring in a particular sequence or state, but that protocol is not explicitly represented in the API, state model, or type system.

9. **Leaky abstraction**
   Users of an abstraction must know details of its internal representation or implementation in order to use it correctly, making internal changes unnecessarily risky.

10. **Speculative abstraction**
    An abstraction, indirection, configuration mechanism, or extension point exists primarily to support hypothetical future variation rather than variation that the codebase actually needs.


