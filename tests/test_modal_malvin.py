from __future__ import annotations

import json
import subprocess
import sys
from types import SimpleNamespace

import pytest
from click.testing import CliRunner

from toolchain_repos import load_ops_entry, malvin_repo_root

_shim = load_ops_entry("modal_malvin")

split_argv = _shim.split_argv
modal_malvin_cli = _shim.modal_malvin_cli
LaunchCommand = _shim.LaunchCommand
load_lib = _shim.load_lib


@pytest.mark.parametrize(
    ("argv", "ours", "rest"),
    [
        ([], [], []),
        (["--do", "Hello"], [], ["--do", "Hello"]),
        (["--cpu=4", "--gpu", "H100", "--do", "x"], ["--cpu=4", "--gpu", "H100"], ["--do", "x"]),
        (["--cpu=4", "--", "--help"], ["--cpu=4"], ["--help"]),
        (["-h"], ["-h"], []),
        (["--model=grok", "--timeout", "5"], [], ["--model=grok", "--timeout", "5"]),
    ],
)
def test_split_argv(argv: list[str], ours: list[str], rest: list[str]) -> None:
    assert split_argv(argv) == (ours, rest)


def _invoke(monkeypatch: pytest.MonkeyPatch, argv: list[str], exit_code: int = 0):
    calls = []

    def launch(malvin_args, **options):
        calls.append((tuple(malvin_args), options))
        return exit_code

    monkeypatch.setattr(_shim, "load_lib", lambda: SimpleNamespace(launch=launch))
    result = CliRunner().invoke(modal_malvin_cli, argv)
    return result, calls


def test_cli_passes_malvin_args_verbatim(monkeypatch: pytest.MonkeyPatch) -> None:
    result, calls = _invoke(monkeypatch, ["--gpu", "H100", "--timeout=60", "--do", "x", "--help"])
    assert result.exit_code == 0, result.output
    assert calls == [(("--do", "x", "--help"), {"cpu": None, "gpu": "H100", "timeout": 60})]


def test_cli_double_dash_and_exit_code(monkeypatch: pytest.MonkeyPatch) -> None:
    result, calls = _invoke(monkeypatch, ["--cpu=4", "--", "--help"], exit_code=3)
    assert result.exit_code == 3
    assert calls == [(("--help",), {"cpu": 4.0, "gpu": None, "timeout": None})]


def test_cli_rejects_missing_option_value(monkeypatch: pytest.MonkeyPatch) -> None:
    result, calls = _invoke(monkeypatch, ["--gpu"])
    assert result.exit_code == 2
    assert calls == []
    assert isinstance(modal_malvin_cli, LaunchCommand)


def test_cli_no_args_shows_usage(monkeypatch: pytest.MonkeyPatch) -> None:
    result, calls = _invoke(monkeypatch, [])
    assert result.exit_code == 2
    assert calls == []
    assert "Usage: " in result.output and "--timeout" in result.output


def test_help_does_not_import_modal() -> None:
    script = malvin_repo_root() / "ops" / "modal_malvin.py"
    code = (
        "import runpy, sys\n"
        f"sys.argv = [{str(script)!r}, '--help']\n"
        "try:\n"
        f"    runpy.run_path({str(script)!r}, run_name='__main__')\n"
        "except SystemExit as e:\n"
        "    assert e.code == 0, e.code\n"
        "print('modal' in sys.modules)\n"
    )
    out = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, check=True)
    assert out.stdout.strip().splitlines()[-1] == "False"


def test_shim_exposes_modal_app_lazily() -> None:
    pytest.importorskip("modal")
    assert "app" in dir(_shim)
    assert _shim.app is load_lib().app
    with pytest.raises(AttributeError):
        _shim.no_such_name


def _lib():
    pytest.importorskip("modal")
    return load_lib()


def test_malvin_model() -> None:
    lib = _lib()
    assert lib.malvin_model(["--model=grok", "x"]) == "grok"
    assert lib.malvin_model(["--model", "local"]) == "local"
    assert lib.malvin_model(["--do", "x", "--model"]) is None
    assert lib.malvin_cmd(["--do", "x"]) == ["malvin", "--do", "x"]


def test_upload_requests(tmp_path, monkeypatch: pytest.MonkeyPatch) -> None:
    lib = _lib()
    monkeypatch.chdir(tmp_path)
    (tmp_path / "req.md").write_text("hi")
    args, files = lib.upload_requests(["--model=api", "req.md", "missing.md"])
    remote = f"{lib.WORKDIR}/0_req.md"
    assert args == ["--model=api", remote, "missing.md"]
    assert files == {remote: "hi"}


def _event(text: str, event: str = "assistant", direction: str = "in") -> str:
    return json.dumps({"direction": direction, "message": {"event": event, "text": text}})


def test_trace_event_text() -> None:
    lib = _lib()
    assert lib.trace_event_text("not json") == ""
    assert lib.trace_event_text(_event("x", event="thinking")) == ""
    assert lib.trace_event_text(_event("hello")) == "hello"
    assert lib.trace_event_text(_event("x", event="tool")) == "\n"


def test_dm_tail_streams_dm_bodies(tmp_path) -> None:
    lib = _lib()
    trace = tmp_path / "trace.jsonl"
    tail = lib.DmTail(str(trace))
    assert tail.poll() == []
    body = f"pre\n{lib.DM_START}\nline one\n{lib.DM_END}\npost\n"
    trace.write_text(_event(body) + "\n" + _event("par"))
    assert tail.poll() == ["line one\n"]


def _sentinel_fixture() -> dict:
    path = malvin_repo_root() / "tests" / "fixtures" / "sentinel_lines.json"
    return json.loads(path.read_text(encoding="utf-8"))


def test_sentinel_markers_match_shared_fixture() -> None:
    lib = _lib()
    markers = _sentinel_fixture()["markers"]
    assert lib.DM_START == markers["dm_start"]
    assert lib.DM_END == markers["dm_end"]


def test_sentinel_rule_matches_shared_fixture() -> None:
    lib = _lib()
    fixture = _sentinel_fixture()
    for case in fixture["cases"]:
        marker = fixture["markers"][case["marker"]]
        assert lib.is_sentinel_line(case["line"], marker) is case["matches"], case


def test_dm_tail_accepts_whitespace_around_markers(tmp_path) -> None:
    lib = _lib()
    trace = tmp_path / "trace.jsonl"
    tail = lib.DmTail(str(trace))
    body = f"  {lib.DM_START}\r\nspaced\n {lib.DM_END} \n"
    trace.write_text(_event(body) + "\n" + _event("par"))
    assert tail.poll() == ["spaced\n"]
