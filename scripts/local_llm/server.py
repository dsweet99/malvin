#!/usr/bin/env python3
"""OpenAI-compatible HTTP sidecar for malvin `local:` models (MLX / Apple Silicon)."""

from __future__ import annotations

import argparse
import json
import sys
import threading
import time
import traceback
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any


def load_model(loader: str, model_dir: Path):
    if loader == "jang":
        from jang_tools.loader import load_jang_model

        return load_jang_model(str(model_dir))
    from mlx_lm import load

    return load(str(model_dir))


def messages_to_prompt(tokenizer: Any, messages: list[dict[str, str]]) -> str:
    if hasattr(tokenizer, "apply_chat_template"):
        try:
            return tokenizer.apply_chat_template(
                messages,
                tokenize=False,
                add_generation_prompt=True,
                enable_thinking=False,
            )
        except TypeError:
            return tokenizer.apply_chat_template(
                messages,
                tokenize=False,
                add_generation_prompt=True,
            )
    parts = []
    for msg in messages:
        parts.append(f"{msg.get('role', 'user')}: {msg.get('content', '')}")
    parts.append("assistant:")
    return "\n".join(parts)


class ModelState:
    def __init__(self, model_id: str, model_dir: Path, loader: str) -> None:
        self.model_id = model_id
        self.model_dir = model_dir
        self.loader = loader
        self.model = None
        self.tokenizer = None
        self.lock = threading.Lock()
        self.ready = False
        self.load_error: str | None = None

    def ensure_loaded(self) -> None:
        with self.lock:
            if self.ready:
                return
            if self.load_error is not None:
                raise RuntimeError(self.load_error)
            try:
                print(f"loading model={self.model_dir} loader={self.loader}", flush=True)
                t0 = time.perf_counter()
                self.model, self.tokenizer = load_model(self.loader, self.model_dir)
                print(f"loaded in {time.perf_counter() - t0:.2f}s", flush=True)
                self.ready = True
            except Exception as exc:  # noqa: BLE001 — surface any load failure to HTTP clients
                self.load_error = f"{exc}\n{traceback.format_exc()}"
                raise

    def complete(self, messages: list[dict[str, str]], max_tokens: int) -> str:
        self.ensure_loaded()
        from mlx_lm import generate
        from mlx_lm.sample_utils import make_sampler

        assert self.model is not None
        assert self.tokenizer is not None
        prompt = messages_to_prompt(self.tokenizer, messages)
        with self.lock:
            return generate(
                self.model,
                self.tokenizer,
                prompt=prompt,
                max_tokens=max_tokens,
                sampler=make_sampler(temp=0.0),
                verbose=False,
            )


def make_handler(state: ModelState):
    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def log_message(self, fmt: str, *args: Any) -> None:
            sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))

        def _read_json(self) -> dict[str, Any]:
            length = int(self.headers.get("Content-Length", "0"))
            raw = self.rfile.read(length) if length else b"{}"
            return json.loads(raw.decode("utf-8") or "{}")

        def _send(self, code: int, payload: dict[str, Any]) -> None:
            body = json.dumps(payload).encode("utf-8")
            self.send_response(code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Connection", "close")
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self) -> None:  # noqa: N802
            path = self.path.split("?", 1)[0]
            if path in ("/health", "/v1/health"):
                self._send(200, {"ok": True, "model": state.model_id, "ready": state.ready})
                return
            if path in ("/v1/models", "/models"):
                self._send(
                    200,
                    {
                        "object": "list",
                        "data": [
                            {
                                "id": state.model_id,
                                "object": "model",
                                "owned_by": "malvin-local",
                            }
                        ],
                    },
                )
                return
            self._send(404, {"error": {"message": f"not found: {path}"}})

        def do_POST(self) -> None:  # noqa: N802
            path = self.path.split("?", 1)[0]
            if path not in ("/v1/chat/completions", "/chat/completions"):
                self._send(404, {"error": {"message": f"not found: {path}"}})
                return
            try:
                req = self._read_json()
                messages = req.get("messages") or []
                if not isinstance(messages, list) or not messages:
                    self._send(400, {"error": {"message": "messages required"}})
                    return
                normalized = []
                for msg in messages:
                    if not isinstance(msg, dict):
                        continue
                    normalized.append(
                        {
                            "role": str(msg.get("role", "user")),
                            "content": str(msg.get("content", "")),
                        }
                    )
                max_tokens = int(req.get("max_tokens") or req.get("max_completion_tokens") or 1024)
                content = state.complete(normalized, max_tokens=max_tokens)
                self._send(
                    200,
                    {
                        "id": f"chatcmpl-{uuid.uuid4().hex[:12]}",
                        "object": "chat.completion",
                        "model": state.model_id,
                        "choices": [
                            {
                                "index": 0,
                                "message": {"role": "assistant", "content": content},
                                "finish_reason": "stop",
                            }
                        ],
                        "usage": {
                            "prompt_tokens": 0,
                            "completion_tokens": 0,
                            "total_tokens": 0,
                        },
                    },
                )
            except Exception as exc:  # noqa: BLE001
                self._send(500, {"error": {"message": str(exc)}})

    return Handler


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, required=True)
    parser.add_argument("--model-dir", type=Path, required=True)
    parser.add_argument("--model-id", required=True)
    parser.add_argument("--loader", default="mlx_lm", choices=("mlx_lm", "jang"))
    parser.add_argument(
        "--preload",
        action="store_true",
        help="Load weights before accepting requests (default: lazy on first completion)",
    )
    args = parser.parse_args()

    model_dir = args.model_dir.expanduser().resolve()
    if not model_dir.is_dir():
        print(f"model dir missing: {model_dir}", file=sys.stderr)
        return 1

    state = ModelState(args.model_id, model_dir, args.loader)
    # Become healthy for listing before weights finish loading.
    if args.preload:
        try:
            state.ensure_loaded()
        except Exception as exc:  # noqa: BLE001
            print(f"preload failed: {exc}", file=sys.stderr)
            return 1

    handler = make_handler(state)
    server = ThreadingHTTPServer((args.host, args.port), handler)
    print(
        f"malvin local sidecar listening on http://{args.host}:{args.port}/v1 model={args.model_id}",
        flush=True,
    )
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
