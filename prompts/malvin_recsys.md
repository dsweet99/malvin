
Understand VISION.md, the codebase, and notes from the user (./malvin-recsys/notes.md).

Propose a PR-sized project that you think will delight the user.
- Feature
- Bug fix
- Performance improvement
- UX improvement
- Anything, as long as it's consistent with VISION.md
Be creative.


Branch from `malvin/proposals` to a new branch called `malvin/some-meaningful-name`.

Implement your proposal
 - New features should go behind a feature flag, with the flag off. We're
   using continuous-integration, dark-launch style.
 - Design with the knowledge that there's a chance the user with
   dislike your change and, later, as to revert it. IOW, be modular
   and loosely coupled. But, you know, that's generally good design, anyway.
 - Get all gates to pass.
 
QA-test it until you find a bug. Fix the bug. Repeat until the
 implementation is rock solid.

Write a brief description of your new project in `./malvin-recsys/reports/MRnnnn-some-meaningful-name.md` (the
 same name as the branch). `nnnn` should expand to an incrementing id number: 0001, 0002, 0003, ... Put a
 title at the top of the report. Follow it with a tl;dr. Then
 a short (max 5 sentences) summary. Then the full description.

Commit it with a good commit message. Then merge it back into `malvin/proposals`!
