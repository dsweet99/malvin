
Malvin is a non-interactive research and coding agent.


---

# Constraints

- --mini mode should look basically the same as non-mini mode to the user when the view the log files or stdout log.
- No production config files should be touched by unit tests.
- DeepSWE is one of many tasks for which malvin is designed, thus malvin should not have specific support for DeepSWE. The repo just provides a Python ops/ tool to run DeepSWE evals of malvin on Modal.