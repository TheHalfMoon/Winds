#!/usr/bin/env python3
"""T143 native Tauri cold-launch and idle resource qualification on Linux."""
from __future__ import annotations

import argparse
import json
import os
import re
import select
import shutil
import signal
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any

READY = re.compile(rb"WINDS_T143_READY_MS=([0-9]+(?:\.[0-9]+)?)")


def percentile(values: list[float], percent: int) -> float:
    ordered = sorted(values)
    if not ordered:
        raise ValueError("cannot summarize empty samples")
    rank = max(1, (len(ordered) * percent + 99) // 100)
    return ordered[min(rank - 1, len(ordered) - 1)]


def summary(values: list[float]) -> dict[str, Any]:
    return {
        "sample_count": len(values),
        "p50_ms": round(percentile(values, 50), 4),
        "p95_ms": round(percentile(values, 95), 4),
        "max_ms": round(max(values), 4),
        "raw_ms": [round(value, 4) for value in values],
    }


def launch_env(home: str) -> dict[str, str]:
    env = os.environ.copy()
    env.update({
        "WINDS_HOME": home,
        "GDK_BACKEND": "x11",
        "WEBKIT_DISABLE_DMABUF_RENDERER": "1",
    })
    return env


def start(binary: str, home: str) -> tuple[subprocess.Popen[bytes], int]:
    started_ns = time.perf_counter_ns()
    proc = subprocess.Popen(
        [binary],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env=launch_env(home),
        start_new_session=True,
    )
    return proc, started_ns


def await_ready(proc: subprocess.Popen[bytes], started_ns: int, timeout: float = 15.0) -> tuple[float, float, bytes]:
    assert proc.stdout is not None
    captured = bytearray()
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            remainder = proc.stdout.read() or b""
            captured.extend(remainder)
            raise RuntimeError(f"Winds exited before T143 readiness marker: rc={proc.returncode} output={captured[-4000:]!r}")
        remaining = max(0.0, deadline - time.monotonic())
        readable, _, _ = select.select([proc.stdout.fileno()], [], [], min(0.25, remaining))
        if not readable:
            continue
        chunk = os.read(proc.stdout.fileno(), 4096)
        if not chunk:
            continue
        captured.extend(chunk)
        match = READY.search(captured)
        if match:
            external_ms = (time.perf_counter_ns() - started_ns) / 1_000_000.0
            internal_ms = float(match.group(1))
            return external_ms, internal_ms, bytes(captured)
    raise RuntimeError(f"T143 readiness marker missing after {timeout}s: {captured[-4000:]!r}")


def terminate(proc: subprocess.Popen[bytes]) -> None:
    if proc.poll() is not None:
        return
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        proc.wait(timeout=3)
        return
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    proc.wait(timeout=3)


def descendants(root: int) -> set[int]:
    seen: set[int] = set()
    pending = [root]
    while pending:
        pid = pending.pop()
        if pid in seen or not Path(f"/proc/{pid}").exists():
            continue
        seen.add(pid)
        children = Path(f"/proc/{pid}/task/{pid}/children")
        try:
            pending.extend(int(value) for value in children.read_text().split())
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
    return seen


def proc_stat(pid: int) -> tuple[int, int] | None:
    try:
        stat = Path(f"/proc/{pid}/stat").read_text()
        end = stat.rfind(")")
        fields = stat[end + 2 :].split()
        ticks = int(fields[11]) + int(fields[12])
        rss_pages = int(Path(f"/proc/{pid}/statm").read_text().split()[1])
        return ticks, rss_pages
    except (FileNotFoundError, PermissionError, ProcessLookupError, ValueError, IndexError):
        return None


def proc_name(pid: int) -> str:
    try:
        return Path(f"/proc/{pid}/comm").read_text().strip()
    except (FileNotFoundError, PermissionError, ProcessLookupError):
        return "unavailable"


def idle_campaign(proc: subprocess.Popen[bytes], seconds: float, settle: float) -> dict[str, Any]:
    page_size = os.sysconf("SC_PAGE_SIZE")
    clk_tck = os.sysconf("SC_CLK_TCK")
    time.sleep(settle)
    started = time.monotonic()
    previous_ticks: dict[int, int] = {}
    accumulated_ticks = 0
    max_rss = 0
    max_process_count = 0
    observed_names: set[str] = set()
    samples: list[dict[str, Any]] = []

    while time.monotonic() - started < seconds:
        pids = descendants(proc.pid)
        rss = 0
        for pid in pids:
            observed_names.add(proc_name(pid))
            stat = proc_stat(pid)
            if stat is None:
                continue
            ticks, pages = stat
            rss += pages * page_size
            prior = previous_ticks.get(pid)
            if prior is not None and ticks >= prior:
                accumulated_ticks += ticks - prior
            previous_ticks[pid] = ticks
        max_rss = max(max_rss, rss)
        max_process_count = max(max_process_count, len(pids))
        samples.append({
            "elapsed_ms": round((time.monotonic() - started) * 1000.0, 3),
            "rss_bytes": rss,
            "process_count": len(pids),
        })
        time.sleep(0.25)

    elapsed = time.monotonic() - started
    cpu_seconds = accumulated_ticks / float(clk_tck)
    cpu_percent_one_core = cpu_seconds / elapsed * 100.0
    return {
        "duration_ms": round(elapsed * 1000.0, 3),
        "settle_ms": round(settle * 1000.0, 3),
        "cpu_percent_one_logical_core": round(cpu_percent_one_core, 5),
        "cpu_scope": "Tauri host plus descendant WebKitGTK processes; no terminal/agent action executed",
        "rss_max_bytes": max_rss,
        "rss_max_mib": round(max_rss / (1024 * 1024), 3),
        "rss_scope": "Tauri host plus descendant WebKitGTK processes; child agents/terminals absent by fixture design",
        "max_process_count": max_process_count,
        "observed_process_names": sorted(name for name in observed_names if name),
        "raw_samples": samples,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--launches", type=int, default=20)
    parser.add_argument("--idle-seconds", type=float, default=60.0)
    parser.add_argument("--settle-seconds", type=float, default=2.0)
    args = parser.parse_args()

    binary = str(Path(args.binary).resolve())
    if not Path(binary).is_file():
        raise RuntimeError(f"T143 binary missing: {binary}")

    external_samples: list[float] = []
    internal_samples: list[float] = []
    for index in range(args.launches):
        home = tempfile.mkdtemp(prefix=f"winds-t143-launch-{index:02d}-")
        proc: subprocess.Popen[bytes] | None = None
        try:
            proc, started_ns = start(binary, home)
            external_ms, internal_ms, _ = await_ready(proc, started_ns)
            external_samples.append(external_ms)
            internal_samples.append(internal_ms)
        finally:
            if proc is not None:
                terminate(proc)
            shutil.rmtree(home, ignore_errors=True)

    idle_home = tempfile.mkdtemp(prefix="winds-t143-idle-")
    idle_proc: subprocess.Popen[bytes] | None = None
    try:
        idle_proc, started_ns = start(binary, idle_home)
        idle_external_ms, idle_internal_ms, _ = await_ready(idle_proc, started_ns)
        idle = idle_campaign(idle_proc, args.idle_seconds, args.settle_seconds)
    finally:
        if idle_proc is not None:
            terminate(idle_proc)
        shutil.rmtree(idle_home, ignore_errors=True)

    result = {
        "schema": "winds-t143-native-performance-v1",
        "binary": binary,
        "launch": {
            "external_process_to_useful_shell": summary(external_samples),
            "host_internal_to_ready_navigation": summary(internal_samples),
            "sample_count": args.launches,
            "measurement_boundary": "process start to benchmark-only populated two-Session shell after two requestAnimationFrame boundaries; readiness is observed by feature-gated Tauri page-load hook without adding IPC commands",
        },
        "idle_launch": {"external_ms": round(idle_external_ms, 4), "internal_ms": round(idle_internal_ms, 4)},
        "idle": idle,
    }
    checks = {
        "cold_launch_p95_le_1500_ms": result["launch"]["external_process_to_useful_shell"]["p95_ms"] <= 1500.0,
        "launch_samples_ge_20": args.launches >= 20,
        "idle_cpu_le_2_percent_one_core": idle["cpu_percent_one_logical_core"] <= 2.0,
        "renderer_host_idle_rss_le_300_mib": idle["rss_max_bytes"] <= 300 * 1024 * 1024,
    }
    result["checks"] = checks
    Path(args.output).write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print("T143_NATIVE_JSON=" + json.dumps(result, sort_keys=True))
    failed = [name for name, passed in checks.items() if not passed]
    if failed:
        print(f"T143 native qualification failed: {failed}", file=os.sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
