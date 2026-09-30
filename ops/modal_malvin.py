#!/usr/bin/env python3
"""Click CLI for running malvin jobs on Modal — implementation in ``src/python/modal_malvin_lib.py``.

Runs malvin against a local vLLM model or an API-provided model.

Everything persistent lives in one Modal Volume, `malvin-data`, mounted at /data:
    /data/malvin_home/logs   malvin run logs; /root/.malvin_home/logs links here
    /data/hf            Hugging Face cache (model weights)
    /data/vllm          vLLM compile cache
    /data/vllm_logs     vLLM server logs, one file per container

Usage (from the repository root):
    modal run ops/modal_malvin.py::download           # fetch weights into the volume (CPU only, once)
    modal deploy ops/modal_malvin.py                  # deploy the app (after source changes)
    ./ops/modal_malvin.py --do Hello                  # `malvin --model=api --do Hello`, CPU container
    ./ops/modal_malvin.py --model=grok request.md     # `malvin --model=grok request.md`
    ./ops/modal_malvin.py --model=local --do Hello    # vLLM on the default A100-40GB
    ./ops/modal_malvin.py --gpu=H100 --timeout=3600 --do Hello   # vLLM on an H100, 1 h limit
    ./ops/modal_malvin.py --cpu=4 -- --help           # `--` ends the launch options
    modal volume ls malvin-data malvin_home/logs      # browse persisted malvin logs

Leading --cpu, --gpu, and --timeout (seconds) override the container's resources for
this call; everything after them is passed to malvin unchanged. Without `--model`,
malvin gets `--model=local` when --gpu is given and `--model=api` otherwise. Model
`local` runs in LocalMalvin (vLLM); any other model runs in ApiMalvin. Container
nicknames: api, local, grok (OpenRouter). Arguments naming existing local `.md`
files are uploaded into the container's work directory and rewritten to point there
(later local edits are not seen, so `--watch` has no effect). While malvin runs, the container tails `trace.jsonl` in each new run directory and
streams every DM body line to local stdout as the model writes it. malvin's own
stdout is discarded (its logs are on the Volume); its stderr and a timing line go
to local stderr when the job ends. Pressing Ctrl-C cancels the call and stops malvin.

Containers stay warm for SCALEDOWN_S after a job, so later jobs skip startup.
LocalMalvin uses a CPU+GPU memory snapshot of a sleeping vLLM server, so cold
starts restore it instead of reloading weights; the first few cold starts after a
deploy build the snapshot and take minutes. `modal deploy` does not retire warm
containers; stop them with `modal container stop -y <id>` to pick up new code.
The API class reads OPENROUTER_API_KEY from the Modal Secret `malvin-openrouter`.

The library (and with it `modal`) is imported only when a job is launched or when
`modal run`/`modal deploy` looks up the app, so `--help`, a bare invocation (which
prints usage), and usage errors return quickly.
"""

from __future__ import annotations

import importlib
import sys
from pathlib import Path
from types import ModuleType
from typing import Any

import click

_src = Path(__file__).resolve().parents[1] / "src" / "python"
if str(_src) not in sys.path:
    sys.path.insert(0, str(_src))

LAUNCH_OPTIONS = ("--cpu", "--gpu", "--timeout")
MODAL_EXPORTS = ("app", "download", "ApiMalvin", "LocalMalvin")


def load_lib() -> ModuleType:
    return importlib.import_module("_ops_bootstrap").load_library("modal_malvin_lib")


def __getattr__(name: str) -> Any:
    if name in MODAL_EXPORTS:
        return getattr(load_lib(), name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def __dir__() -> list[str]:
    return sorted({*globals(), *MODAL_EXPORTS})


def split_argv(argv: list[str]) -> tuple[list[str], list[str]]:
    """Split leading launch options (and an optional `--`) from the malvin arguments."""
    i = 0
    while i < len(argv) and argv[i] != "--":
        name = argv[i].split("=", 1)[0]
        if name in ("-h", "--help"):
            i += 1
        elif name in LAUNCH_OPTIONS:
            i += 1 if "=" in argv[i] else 2
        else:
            return argv[:i], argv[i:]
    return argv[:i], argv[i + 1 :]


class LaunchCommand(click.Command):
    """Parse only the leading launch options; everything after them goes to malvin verbatim."""

    def parse_args(self, ctx: click.Context, args: list[str]) -> list[str]:
        ours, malvin_args = split_argv(args)
        return super().parse_args(ctx, [*ours, "--", *malvin_args] if malvin_args else ours)


@click.command(
    cls=LaunchCommand,
    context_settings={"help_option_names": ["-h", "--help"]},
    no_args_is_help=True,
    options_metavar="[--cpu N] [--gpu TYPE] [--timeout SECONDS] [--]",
)
@click.option("--cpu", type=float, help="CPU cores (default: 2 for ApiMalvin, 8 for LocalMalvin)")
@click.option("--gpu", help="Modal GPU spec, e.g. A100-40GB, A100-80GB, H100, L40S:2")
@click.option("--timeout", type=int, help="job timeout in seconds (default: 24 hours)")
@click.argument("malvin_args", nargs=-1, type=click.UNPROCESSED)
@click.pass_context
def modal_malvin_cli(ctx: click.Context, malvin_args: tuple[str, ...], **launch_options: Any) -> None:
    """Run malvin on Modal. Arguments after the launch options go to malvin unchanged."""
    ctx.exit(load_lib().launch(malvin_args, **launch_options))


__all__ = [
    "LaunchCommand",
    "load_lib",
    "modal_malvin_cli",
    "split_argv",
]

if __name__ == "__main__":
    modal_malvin_cli()
