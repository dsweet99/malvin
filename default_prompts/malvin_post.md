

Forked rich
Branched from rich:46cebbb032f920eb096efbaf23cdc6fe9dd541f7

constraints:
statements_per_function <= 25
max_indentation_depth <= 4
cycle_size <= 0

All other metrics clamped to current values.

Starting maximum values:
statements_per_function = 127
max_indentation_depth = 13
cycle_size = 53

o|TIMING: wall = 78620.8s llm_wait = 75415.2s tool_calls = 54004.6s implement = 75415.2s
21 hours
69% of time in tools calls
- very long-running unit tests in test_examples_progress.py
    - They passed, though.
    - All other unit tests passed 

# Good
- All kiss metric constraints satisfied.
 - Large dependency cycle gone.
- All unit tests pass
- All glances unit tests pass (except the four that didn't pass with official rich, either)
 - glances is a monitoring application that makes heavy use of rich.

rich                                     15.0.0
ERROR tests/test_webui.py::test_screenshot - selenium.common.exceptions.SessionNotCreatedException: Message: session not created
ERROR tests/test_webui.py::test_loading_time - selenium.common.exceptions.SessionNotCreatedException: Message: session not created
ERROR tests/test_webui.py::test_title - selenium.common.exceptions.SessionNotCreatedException: Message: session not created
ERROR tests/test_webui.py::test_plugins - selenium.common.exceptions.SessionNotCreatedException: Message: session not created

rich                                     15.0.0      /home/dsweet/Projects/experiments/exp-rich-tidy
ERROR tests/test_webui.py::test_screenshot - selenium.common.exceptions.SessionNotCreatedException: Message: session not created
ERROR tests/test_webui.py::test_loading_time - selenium.common.exceptions.SessionNotCreatedException: Message: session not created
ERROR tests/test_webui.py::test_title - selenium.common.exceptions.SessionNotCreatedException: Message: session not created
ERROR tests/test_webui.py::test_plugins - selenium.common.exceptions.SessionNotCreatedException: Message: session not created


￼

# Bad
- one very long-running unit test file (test_examples_progress.py); passed, though
- some simplistic tests, created, apparently, to meet the high per-file test threshold (85%): test_kiss_coverage.py
    - not sure if this is bad, though; the do exercise code and assert; better than nothing
- used importlib in test files to hide dependencies from kiss's imported_names_per_file metric. It could have just used more files.
- 70% of time in tool calls. Maybe overly thorough with pytest. In practice, I often use testmon to speed things up. This experiment did not used tesmon.

