
import json
import os
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import urllib.request

import modal

APP_NAME = "malvin"
MODEL_ID = "RedHatAI/Qwen3.8-27B-INT4"
SERVED_NAME = "qwen3.8-27b"
API_MODEL = "pi:openrouter/anthropic/claude-haiku-4.5"
LOCAL_MODEL = f"pi:local/llamacpp/{SERVED_NAME}"
GROK_MODEL = "pi:openrouter/x-ai/grok-4.7"
MAX_MODEL_LEN = 131072
PORT = 8080
SCALEDOWN_S = 600
JOB_TIMEOUT_S = 24 * 60 * 60
DEFAULT_ARGS = ("--do", "Hello")
STDERR_TAIL = 4000
POLL_S = 0.5
DM_START = "__MALVIN_DM_START__"
DM_END = "__MALVIN_DM_END__"
ASCII_WHITESPACE = " \t\n\x0c\r"
DATA = "/data"
MALVIN_HOME = "/root/.malvinconf"
LOGS = f"{DATA}/malvin_home/logs"
VLLM_LOGS = f"{DATA}/vllm_logs"
VLLM_LOCAL_LOG = "/tmp/vllm.log"
WORKDIR = "/root/work"
MALVIN_SRC_TAR = "/tmp/malvin-src.tar"
REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

if modal.is_local():
    subprocess.run(
        ["git", "-C", REPO_ROOT, "archive", "--format=tar", "-o", MALVIN_SRC_TAR, "HEAD"],
        check=True,
    )

BUILD_DEPS = (
    "build-essential", "pkg-config", "libssl-dev", "git", "curl", "ca-certificates", "xz-utils",
)
NODE_DIST = "node-v22.23.2-linux-x64"
PI_PREFIX = "/opt/pi"
PI_PACKAGE = "@earendil-works/pi-coding-agent@1.0.2"
INSTALL_PI = (
    f"curl -fsSL https://nodejs.org/dist/v22.23.2/{NODE_DIST}.tar.xz | tar -xJ -C /opt"
    f" && ln -sf /opt/{NODE_DIST}/bin/node /usr/local/bin/node"
    f" && /opt/{NODE_DIST}/bin/npm install --prefix {PI_PREFIX} {PI_PACKAGE}"
)
PI_ENV = {
    "MALVIN_PI": f"{PI_PREFIX}/node_modules/@earendil-works/pi-coding-agent/dist/bundle/rpc-entry.js",
    "MALVIN_NODE": "/usr/local/bin/node",
}
RUSTUP = "curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.96.0"
BUILD_MALVIN = (
    "mkdir -p /opt/malvin && tar -xf /opt/malvin-src.tar -C /opt/malvin"
    " && export PATH=/root/.cargo/bin:$PATH && cd /opt/malvin"
    " && cargo install --path . --locked --root /usr/local"
)
LINK_HOME = f"rm -rf {MALVIN_HOME} && mkdir -p {MALVIN_HOME} && ln -s {LOGS} {MALVIN_HOME}/logs"

api_image = (
    modal.Image.debian_slim(python_version="3.12")
    .apt_install(*BUILD_DEPS)
    .env({"MALVIN_DISABLE_LLD": "1", **PI_ENV})
    .add_local_file(MALVIN_SRC_TAR, "/opt/malvin-src.tar", copy=True)
    .run_commands(
        INSTALL_PI,
        f"{RUSTUP} && {BUILD_MALVIN}"
        " && rm -rf /root/.cargo /root/.rustup /opt/malvin /opt/malvin-src.tar",
        "malvin --version",
        LINK_HOME,
    )
)

vllm_image = (
    modal.Image.from_registry("vllm/vllm-openai:v0.30.0", add_python="3.12")
    .entrypoint([])
    .pip_install("huggingface_hub")
    .apt_install(*BUILD_DEPS)
    .run_commands(RUSTUP, INSTALL_PI)
    .env({"MALVIN_DISABLE_LLD": "1", **PI_ENV, "HF_HOME": f"{DATA}/hf", "VLLM_CACHE_ROOT": f"{DATA}/vllm"})
    .add_local_file(MALVIN_SRC_TAR, "/opt/malvin-src.tar", copy=True)
    .run_commands(BUILD_MALVIN, "malvin --version", LINK_HOME)
)

data = modal.Volume.from_name("malvin-data", create_if_missing=True)
volumes = {DATA: data}

app = modal.App(APP_NAME)


@app.function(image=vllm_image, volumes=volumes, timeout=60 * 60, cpu=4, memory=8192)
def download():
    from huggingface_hub import snapshot_download

    path = snapshot_download(MODEL_ID)
    data.commit()
    return path


def _prepare_home():
    os.makedirs(LOGS, exist_ok=True)
    with open(os.path.join(MALVIN_HOME, "local_llms.json"), "w") as f:
        json.dump(
            {
                "models": [
                    {
                        "id": f"llamacpp/{SERVED_NAME}",
                        "source": f"hf:{MODEL_ID}",
                        "notes": f"vLLM on a Modal GPU, 127.0.0.1:{PORT}",
                        "context_size": MAX_MODEL_LEN,
                    }
                ]
            },
            f,
            indent=2,
        )
    with open(os.path.join(MALVIN_HOME, "config.toml"), "w") as f:
        f.write(
            "mem_limit_gb = 3\n\n"
            f'[agent]\nmodel = "{API_MODEL}"\n\n'
            f'[nicknames]\napi = "{API_MODEL}"\nlocal = "{LOCAL_MODEL}"\ngrok = "{GROK_MODEL}"\n'
        )
    os.makedirs(WORKDIR, exist_ok=True)
    subprocess.run(["git", "init", "-q", WORKDIR], check=True)


def _run_dirs():
    hashes = [os.path.join(LOGS, h) for h in os.listdir(LOGS)]
    return {os.path.join(h, r) for h in hashes if os.path.isdir(h) for r in os.listdir(h)}


def _count_runs():
    return len(_run_dirs())


def trace_event_text(raw):
    try:
        event = json.loads(raw)
    except ValueError:
        return ""
    message = event.get("message") if isinstance(event, dict) else None
    if not isinstance(message, dict) or message.get("event") == "thinking":
        return ""
    if message.get("event") == "assistant" and event.get("direction") == "in":
        return str(message.get("text", ""))
    return "\n"


def is_sentinel_line(line, marker):
    return line.strip(ASCII_WHITESPACE) == marker


class DmTail:
    def __init__(self, path):
        self.path = path
        self.offset = 0
        self.partial = b""
        self.text = ""
        self.inside = False

    def poll(self):
        if not os.path.exists(self.path):
            return []
        with open(self.path, "rb") as f:
            f.seek(self.offset)
            chunk = f.read()
        self.offset += len(chunk)
        *records, self.partial = (self.partial + chunk).split(b"\n")
        self.text += "".join(trace_event_text(r.decode("utf-8", "replace")) for r in records)
        return self._drain()

    def _drain(self):
        *lines, self.text = self.text.split("\n")
        body = []
        for line in lines:
            if is_sentinel_line(line, DM_END if self.inside else DM_START):
                self.inside = not self.inside
            elif self.inside:
                body.append(line + "\n")
        return body


def _new_dm_lines(tails, runs_before):
    for run in sorted(_run_dirs() - runs_before - tails.keys(), key=os.path.basename):
        tails[run] = DmTail(os.path.join(run, "trace.jsonl"))
    return [line for run in sorted(tails, key=os.path.basename) for line in tails[run].poll()]


def malvin_cmd(args):
    return ["malvin", *args]


def _stop_process_group(proc):
    if proc.poll() is None:
        os.killpg(proc.pid, signal.SIGTERM)
        try:
            proc.wait(timeout=60)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait()


def _save_vllm_log(job):
    if job.server_log is None:
        return
    os.makedirs(VLLM_LOGS, exist_ok=True)
    job.server_log.flush()
    shutil.copyfile(VLLM_LOCAL_LOG, os.path.join(VLLM_LOGS, job.log_name))


def _stream_malvin(job, cmd):
    cold = job.jobs == 0
    job.jobs += 1
    runs_before = _run_dirs()
    tails = {}
    print("$", shlex.join(cmd), flush=True)
    t0 = time.time()
    with tempfile.TemporaryFile("w+") as err:
        proc = subprocess.Popen(
            cmd, cwd=WORKDIR, stdout=subprocess.DEVNULL, stderr=err, text=True, start_new_session=True
        )
        try:
            running = True
            while running:
                running = proc.poll() is None
                for line in _new_dm_lines(tails, runs_before):
                    print(line, end="", flush=True)
                    yield line
                if running:
                    time.sleep(POLL_S)
        finally:
            _stop_process_group(proc)
            _save_vllm_log(job)
            data.commit()
        err.seek(0)
        stderr = err.read()
    print(stderr, end="", flush=True)
    yield {
        "exit_code": proc.returncode,
        "stderr": stderr[-STDERR_TAIL:],
        "cold": cold,
        "startup_s": round(job.enter_s, 1),
        "restored_from_snapshot": job.restored,
        "malvin_s": round(time.time() - t0, 1),
        "runs_before": len(runs_before),
        "runs_after": _count_runs(),
    }


class _Job:
    enter_s = 0.0
    jobs = 0
    restored = None
    server_log = None

    @modal.method()
    def run(self, args: list[str], files: dict[str, str] | None = None):
        for path, text in (files or {}).items():
            with open(path, "w") as f:
                f.write(text)
        yield from _stream_malvin(self, malvin_cmd(args or list(DEFAULT_ARGS)))


@app.cls(
    image=api_image,
    volumes=volumes,
    secrets=[modal.Secret.from_name("malvin-openrouter")],
    cpu=2,
    memory=4096,
    timeout=JOB_TIMEOUT_S,
    scaledown_window=SCALEDOWN_S,
)
class ApiMalvin(_Job):
    @modal.enter()
    def start(self):
        t0 = time.time()
        _prepare_home()
        self.enter_s = time.time() - t0


def _wait_for_server(proc, deadline_s):
    start = time.time()
    while time.time() - start < deadline_s:
        if proc.poll() is not None:
            raise RuntimeError(f"vLLM exited early with code {proc.returncode}")
        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{PORT}/v1/models", timeout=5) as r:
                return json.load(r)
        except Exception:
            time.sleep(1)
    raise TimeoutError("vLLM did not become ready")


def _vllm_post(path, body=None):
    req = urllib.request.Request(
        f"http://127.0.0.1:{PORT}{path}",
        data=json.dumps(body or {}).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=600) as r:
        return r.read()


@app.cls(
    image=vllm_image.env({"VLLM_SERVER_DEV_MODE": "1", "TORCHINDUCTOR_COMPILE_THREADS": "1"}),
    gpu="A100-40GB",
    volumes=volumes,
    cpu=8,
    memory=32768,
    timeout=JOB_TIMEOUT_S,
    startup_timeout=30 * 60,
    scaledown_window=SCALEDOWN_S,
    enable_memory_snapshot=True,
    experimental_options={"enable_gpu_snapshot": True},
)
class LocalMalvin(_Job):
    @modal.enter(snap=True)
    def start(self):
        t0 = time.time()
        self.server_log = open(VLLM_LOCAL_LOG, "w")
        self.server = subprocess.Popen(
            [
                "vllm", "serve", MODEL_ID,
                "--host", "127.0.0.1",
                "--port", str(PORT),
                "--served-model-name", SERVED_NAME,
                "--max-model-len", str(MAX_MODEL_LEN),
                "--max-num-seqs", "8",
                "--gpu-memory-utilization", "0.92",
                "--language-model-only",
                "--reasoning-parser", "qwen3",
                "--enable-auto-tool-choice",
                "--tool-call-parser", "qwen3_xml",
                "--enable-sleep-mode",
            ],
            stdout=self.server_log,
            stderr=subprocess.STDOUT,
        )
        _wait_for_server(self.server, 30 * 60)
        for _ in range(3):
            _vllm_post(
                "/v1/chat/completions",
                {"model": SERVED_NAME, "messages": [{"role": "user", "content": "Hi"}], "max_tokens": 16},
            )
        _vllm_post("/sleep?level=1")
        self.snap_s = time.time() - t0
        self.snap_end = time.time()

    @modal.enter(snap=False)
    def wake(self):
        t0 = time.time()
        self.restored = t0 - self.snap_end > 30
        _vllm_post("/wake_up")
        _wait_for_server(self.server, 10 * 60)
        _prepare_home()
        self.log_name = time.strftime("%Y%m%d_%H%M%S.log", time.gmtime())
        self.enter_s = time.time() - t0 + (0 if self.restored else self.snap_s)

    @modal.exit()
    def stop(self):
        self.server.terminate()
        self.server.wait(timeout=60)
        _save_vllm_log(self)
        data.commit()


def malvin_model(args):
    for i, arg in enumerate(args):
        if arg.startswith("--model="):
            return arg.split("=", 1)[1]
        if arg == "--model" and i + 1 < len(args):
            return args[i + 1]
    return None


def upload_requests(args):
    files = {}
    remote_args = []
    for arg in args:
        if arg.endswith(".md") and os.path.isfile(arg):
            remote = os.path.join(WORKDIR, f"{len(files)}_{os.path.basename(arg)}")
            with open(arg) as f:
                files[remote] = f.read()
            arg = remote
        remote_args.append(arg)
    return remote_args, files


def launch(malvin_args, *, cpu=None, gpu=None, timeout=None):
    args, files = upload_requests(list(malvin_args) or list(DEFAULT_ARGS))
    model = malvin_model(args)
    if model is None:
        model = "local" if gpu else "api"
        args = [f"--model={model}", *args]
    cls_name = "LocalMalvin" if model in ("local", LOCAL_MODEL) else "ApiMalvin"
    cls = modal.Cls.from_name(APP_NAME, cls_name)
    overrides = {k: v for k, v in {"cpu": cpu, "gpu": gpu, "timeout": timeout}.items() if v is not None}
    if overrides:
        cls = cls.with_options(**overrides)
    t0 = time.time()
    summary = {"exit_code": 1, "stderr": "no summary received from Modal\n"}
    for item in cls().run.remote_gen(args, files):
        if isinstance(item, dict):
            summary = item
        else:
            print(item, end="", flush=True)
    wall = time.time() - t0
    sys.stderr.write(summary.pop("stderr"))
    print(f"TIMING cls={cls_name} model={model} wall_s={wall:.1f} " + " ".join(f"{k}={v}" for k, v in summary.items()), file=sys.stderr)
    return summary["exit_code"]
