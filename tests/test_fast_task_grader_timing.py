from __future__ import annotations

import importlib.util
import shutil
import time
from functools import partial
from pathlib import Path

import pytest


def _burn_cpu(seconds):
    started = time.process_time()
    while time.process_time() - started < seconds:
        pass


def _delayed_events(pause, events):
    for index, event in enumerate(events):
        yield event
        if index == 29_999:
            pause(0.13)


def _apply_with_delay(pause, apply, db, events):
    return apply(db, _delayed_events(pause, events))


def _delayed_roots(pause, roots, correctness_calls):
    calls = 0

    def roots_with_delay(*coefficients):
        nonlocal calls
        calls += 1
        if calls > correctness_calls and (calls - correctness_calls) % 6000 == 0:
            pause(0.05)
        return roots(*coefficients)

    return roots_with_delay


@pytest.mark.parametrize(
    ("pause", "expected_reward"),
    [(time.sleep, 1), (_burn_cpu, 0)],
    ids=["scheduling-delay", "excessive-cpu-work"],
)
def test_journal_grader_replay_budget(tmp_path, monkeypatch, pause, expected_reward):
    path = Path(__file__).resolve().parents[1] / "fast_tasks" / "FT-27" / "grade.py"
    spec = importlib.util.spec_from_file_location("journal_grader", path)
    assert spec is not None and spec.loader is not None
    grader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(grader)
    workspace = tmp_path / "workspace"
    shutil.copytree(grader.default_workspace(), workspace)
    grader._oracle_fix(workspace)
    original_load = grader._load

    def load_with_delay(workspace):
        module = original_load(workspace)
        module.apply = partial(_apply_with_delay, pause, module.apply)
        return module

    monkeypatch.setattr(grader, "_load", load_with_delay)
    assert grader.evaluate(workspace) == expected_reward


@pytest.mark.parametrize(
    ("pause", "expected_reward"),
    [(time.sleep, 1), (_burn_cpu, 0)],
    ids=["scheduling-delay", "excessive-cpu-work"],
)
def test_quadratic_grader_batch_budget(tmp_path, monkeypatch, pause, expected_reward):
    path = Path(__file__).resolve().parents[1] / "fast_tasks" / "FT-24" / "grade.py"
    spec = importlib.util.spec_from_file_location("quadratic_grader", path)
    assert spec is not None and spec.loader is not None
    grader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(grader)
    workspace = tmp_path / "workspace"
    shutil.copytree(grader.default_workspace(), workspace)
    grader._oracle_fix(workspace)
    original_load = grader._load

    def load_with_delay(workspace):
        module = original_load(workspace)
        module.roots = _delayed_roots(pause, module.roots, len(grader.CASES))
        return module

    monkeypatch.setattr(grader, "_load", load_with_delay)
    assert grader.evaluate(workspace) == expected_reward
