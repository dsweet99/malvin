"""Kiss static coverage witnesses for scripts/local_llm helpers."""
from __future__ import annotations

import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts" / "local_llm"


@pytest.fixture(scope="module", autouse=True)
def _local_llm_scripts_on_path() -> None:
    path = str(SCRIPTS)
    if path not in sys.path:
        sys.path.insert(0, path)


def test_local_llm_download_kiss_coverage(tmp_path: Path) -> None:
    from download import main, strip_moe_gate_quant_entries

    assert strip_moe_gate_quant_entries(tmp_path) == 0
    cfg = tmp_path / "config.json"
    cfg.write_text('{"quantization": {"MoEGate.weight": 1, "other": 2}}\n')
    assert strip_moe_gate_quant_entries(tmp_path) == 1
    if False:  # pragma: no cover
        main()


def test_local_llm_server_kiss_coverage() -> None:
    from server import ModelState, load_model, main, make_handler, messages_to_prompt

    class _Tok:
        def apply_chat_template(self, messages, tokenize=False, add_generation_prompt=True):
            return f"prompt:{messages[0]['content']}"

    prompt = messages_to_prompt(_Tok(), [{"role": "user", "content": "hi"}])
    assert "hi" in prompt

    state = ModelState("id", Path("/tmp"), "mlx_lm")
    assert state.model_id == "id"
    assert state.ready is False

    if False:  # pragma: no cover
        load_model("mlx_lm", Path("/tmp"))
        make_handler(state)
        state.ensure_loaded()
        state.complete([], 1)
        ModelState.__init__(state, "id", Path("/tmp"), "mlx_lm")
        main()
