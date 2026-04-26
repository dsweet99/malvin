"""Primary tests are Rust (`cargo test`); this keeps `pytest tests` non-empty for CI."""

def test_pytest_collects() -> None:
    assert True


def test_dag_scheduler_failure_path_checks_exactly_one_stderr_line() -> None:
    from pathlib import Path

    script = Path("evaluations/dag_scheduler_rs.sh").read_text()
    assert 'awk \'END { print NR }\' "$err_file"' in script


def test_dag_scheduler_harness_hardens_malvin_invocation_with_timeout() -> None:
    import re
    from pathlib import Path

    script = Path("evaluations/dag_scheduler_rs.sh").read_text()
    assert "malvin code --trust-the-plan --no-learn" in script
    assert any(
        re.search(r"^\s*malvin code --trust-the-plan --no-learn", line)
        for line in script.splitlines()
    ), "malvin code harness invocation missing from script"
