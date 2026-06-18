"""Modal-free behavioral self-tests for CIDR diagnostic and probe scripts."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

import modal
from click.testing import CliRunner

import diagnose_cidr_dns_modal
import diagnose_cidr_dns_only_modal
import diagnose_cidr_gap_modal
import diagnose_cidr_modal
import diagnose_cidr_observed_modal
import observe_agent_peers_modal
import probe_cidr_connectivity_modal
import probe_cursor_agent_modal
from diagnose_cidr_dns_modal import main as diagnose_cidr_dns_main
from diagnose_cidr_dns_only_modal import main as diagnose_cidr_dns_only_main
from diagnose_cidr_gap_modal import main as diagnose_cidr_gap_main, run_https
from diagnose_cidr_modal import main as diagnose_cidr_main
from diagnose_cidr_observed_modal import main as diagnose_cidr_observed_main
from observe_agent_peers_modal import (
    main as observe_agent_peers_main,
    observe_agent_peers_entry,
    observe_in_sandbox,
)
from probe_cidr_connectivity_modal import main as probe_cidr_connectivity_main
from probe_cursor_agent_modal import (
    main as probe_cursor_agent_main,
    probe_cursor_agent_entry,
    probe_in_sandbox,
)


def _sandbox_proc(stdout: str = "{}", returncode: int = 0) -> MagicMock:
    proc = MagicMock()
    proc.stdout.read.return_value = stdout
    proc.stderr.read.return_value = ""
    proc.wait.return_value = returncode
    return proc


def _sandbox(*procs: MagicMock) -> MagicMock:
    box = MagicMock()
    box.exec.side_effect = list(procs) if len(procs) != 1 else None
    if len(procs) == 1:
        box.exec.return_value = procs[0]
    return box


def _patch_create(*sandboxes: MagicMock):
    return patch.object(modal.Sandbox, "create", side_effect=list(sandboxes))


def _dns_probe_json() -> str:
    return json.dumps(
        {"dns_ips": ["1.2.3.4"], "peer_ips": ["5.6.7.8"], "https": {"ok": True, "status": 200}}
    )


def _task_dir_spec(tmp: str) -> SimpleNamespace:
    task_dir = Path(tmp)
    return SimpleNamespace(task_id="demo-task", tests_dir=task_dir / "tests", dockerfile=None)


def _patch_toolchain(mod: object, spec: SimpleNamespace, fake_image: MagicMock):
    return patch.multiple(
        mod,
        parse_task_dir=MagicMock(return_value=spec),
        materialize_workspace=MagicMock(),
        validate_toolchain_repos=MagicMock(return_value=Path("/malvin")),
        harbor_agent_image=MagicMock(return_value=fake_image),
        cursor_secrets=MagicMock(return_value=[]),
    )


def test_diagnose_cidr_dns_main() -> None:
    open_proc = _sandbox_proc(stdout=_dns_probe_json())
    allow_proc = _sandbox_proc(stdout=_dns_probe_json())
    with _patch_create(_sandbox(open_proc), _sandbox(allow_proc)):
        with patch.object(diagnose_cidr_dns_modal, "cidr_probe_image", return_value=MagicMock()):
            with patch.object(diagnose_cidr_dns_modal, "stream_process_output"):
                diagnose_cidr_dns_main()
    assert open_proc.wait.called and allow_proc.wait.called


def test_diagnose_cidr_dns_only_main() -> None:
    open_proc = _sandbox_proc(stdout='{"api2.cursor.sh": ["1.2.3.4"]}')
    allow_proc = _sandbox_proc(stdout='{"api2.cursor.sh": ["1.2.3.4"]}')
    with _patch_create(_sandbox(open_proc), _sandbox(allow_proc)):
        with patch.object(diagnose_cidr_dns_only_modal, "cidr_probe_image", return_value=MagicMock()):
            with patch.object(diagnose_cidr_dns_only_modal, "stream_process_output"):
                diagnose_cidr_dns_only_main()
    assert open_proc.wait.called and allow_proc.wait.called


def test_diagnose_cidr_gap_main_and_run_https() -> None:
    probe_proc = _sandbox_proc(stdout='["1.2.3.4/32"]')
    allow_proc = _sandbox_proc(stdout='{"ok": true, "status": 200}')
    allow_box = _sandbox(allow_proc)
    with _patch_create(_sandbox(probe_proc), allow_box):
        with patch.object(diagnose_cidr_gap_modal, "cidr_probe_image", return_value=MagicMock()):
            with patch.object(diagnose_cidr_gap_modal, "stream_process_output"):
                diagnose_cidr_gap_main()
                run_https(allow_box)
    assert probe_proc.wait.called and allow_proc.wait.called


def test_diagnose_cidr_main() -> None:
    open_proc = _sandbox_proc(stdout=_dns_probe_json())
    allow_proc = _sandbox_proc(stdout=_dns_probe_json())
    with _patch_create(_sandbox(open_proc), _sandbox(allow_proc)):
        with patch.object(diagnose_cidr_modal, "cidr_probe_image", return_value=MagicMock()):
            with patch.object(diagnose_cidr_modal, "resolve_agent_sandbox_cidrs", return_value=["1.2.3.4/32"]):
                with patch.object(diagnose_cidr_modal, "stream_process_output"):
                    diagnose_cidr_main()
    assert open_proc.wait.called and allow_proc.wait.called


def test_diagnose_cidr_observed_main() -> None:
    peer_proc = _sandbox_proc(stdout='["1.2.3.4"]')
    https_proc = _sandbox_proc(stdout='{"ok": true, "status": 200}')
    with _patch_create(_sandbox(peer_proc), _sandbox(https_proc)):
        with patch.object(diagnose_cidr_observed_modal, "cidr_probe_image", return_value=MagicMock()):
            with patch.object(diagnose_cidr_observed_modal, "stream_process_output"):
                diagnose_cidr_observed_main()
    assert peer_proc.wait.called and https_proc.wait.called


def test_probe_cidr_connectivity_main() -> None:
    proc = _sandbox_proc(stdout="http_code=200\n")
    with _patch_create(_sandbox(proc)):
        with patch.object(probe_cidr_connectivity_modal, "cidr_probe_image", return_value=MagicMock()):
            with patch.object(
                probe_cidr_connectivity_modal,
                "agent_sandbox_network_kwargs",
                return_value={"outbound_cidr_allowlist": ["1.2.3.4/32"]},
            ):
                with patch.object(probe_cidr_connectivity_modal, "stream_process_output"):
                    probe_cidr_connectivity_main()
    proc.wait.assert_called_once()


def test_observe_in_sandbox() -> None:
    proc = _sandbox_proc(stdout='{"peer_ips": ["1.2.3.4"], "peer_ports": [443], "exit_code": 0}')
    with _patch_create(_sandbox(proc)):
        with patch.object(observe_agent_peers_modal, "sandbox_app", return_value=MagicMock()):
            result = observe_in_sandbox(MagicMock(), secrets=[])
    assert result["peer_ips"] == ["1.2.3.4"]
    proc.wait.assert_called_once()


def test_observe_agent_peers_main() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        spec = _task_dir_spec(tmp)
        fake_image = MagicMock()
        patches = _patch_toolchain(observe_agent_peers_modal, spec, fake_image)
        with patches:
            with patch.object(
                observe_agent_peers_modal,
                "observe_in_sandbox",
                return_value={"peer_ips": ["1.2.3.4"], "peer_ports": [443], "exit_code": 0},
            ):
                with patch.object(
                    observe_agent_peers_modal,
                    "resolve_agent_sandbox_cidrs",
                    return_value=["1.2.3.4/32"],
                ):
                    result = CliRunner().invoke(
                        observe_agent_peers_main,
                        ["--task", tmp],
                    )
        assert result.exit_code == 0, result.output


def test_observe_agent_peers_entry() -> None:
    with patch.object(observe_agent_peers_modal, "main") as mock_main:
        observe_agent_peers_entry("--task", "/tmp/task")
    mock_main.main.assert_called_once()


def test_probe_in_sandbox() -> None:
    proc = _sandbox_proc(returncode=0)
    with _patch_create(_sandbox(proc)):
        with patch.object(probe_cursor_agent_modal, "sandbox_app", return_value=MagicMock()):
            with patch.object(probe_cursor_agent_modal, "agent_sandbox_network_kwargs", return_value={}):
                with patch.object(probe_cursor_agent_modal, "stream_process_output"):
                    probe_in_sandbox(MagicMock(), secrets=[], quick=True)
    proc.wait.assert_called_once()


def test_probe_cursor_agent_main() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        spec = _task_dir_spec(tmp)
        fake_image = MagicMock()
        patches = _patch_toolchain(probe_cursor_agent_modal, spec, fake_image)
        with patches:
            with patch.object(probe_cursor_agent_modal, "probe_in_sandbox") as mock_probe:
                result = CliRunner().invoke(
                    probe_cursor_agent_main,
                    ["--task", tmp, "--open-network", "--quick"],
                )
        assert result.exit_code == 0, result.output
        mock_probe.assert_called_once()


def test_probe_cursor_agent_entry() -> None:
    with patch.object(probe_cursor_agent_modal, "main") as mock_main:
        probe_cursor_agent_entry("--task", "/tmp/task", "--quick")
    mock_main.main.assert_called_once()



def run_modal_diag_selftests() -> None:
    """Run all modal diagnostic self-tests (non-pytest entrypoint)."""
    test_diagnose_cidr_dns_main()
    test_diagnose_cidr_dns_only_main()
    test_diagnose_cidr_gap_main_and_run_https()
    test_diagnose_cidr_main()
    test_diagnose_cidr_observed_main()
    test_probe_cidr_connectivity_main()
    test_observe_in_sandbox()
    test_observe_agent_peers_main()
    test_observe_agent_peers_entry()
    test_probe_in_sandbox()
    test_probe_cursor_agent_main()
    test_probe_cursor_agent_entry()
