"""Primary tests are Rust (`cargo test`); this keeps `pytest tests` non-empty for CI."""

def test_pytest_collects() -> None:
    assert True


def test_dag_scheduler_failure_path_checks_exactly_one_stderr_line() -> None:
    from pathlib import Path

    script = Path("evaluations/dag_scheduler_rs.sh").read_text()
    assert 'awk \'END { print NR }\' "$err_file"' in script
