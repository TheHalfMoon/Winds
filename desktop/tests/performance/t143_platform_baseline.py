#!/usr/bin/env python3
"""Measure the same-host minimal-renderer RSS baseline for T143 diagnostics."""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import tempfile
import time
from pathlib import Path
from typing import BinaryIO

import t143_native as native

DEFAULT_READY_WINDOW_PATTERN = r"^Winds \[T143 Platform Baseline\]$"


def ready_window_present(pattern: str) -> bool:
    result = subprocess.run(
        ["xdotool", "search", "--name", pattern],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return result.returncode == 0


def await_ready(
    proc: subprocess.Popen[bytes],
    log: BinaryIO,
    ready_window_pattern: str,
    timeout: float = 15.0,
) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            raise RuntimeError(
                "T143 platform baseline exited before its window became visible: "
                f"rc={proc.returncode} output={native.log_tail(log)!r}"
            )
        if ready_window_present(ready_window_pattern):
            return
        time.sleep(0.05)
    raise RuntimeError(
        f"T143 platform baseline window missing after {timeout}s: {native.log_tail(log)!r}"
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--idle-seconds", type=float, default=10.0)
    parser.add_argument("--ready-window-pattern", default=DEFAULT_READY_WINDOW_PATTERN)
    parser.add_argument("--schema", default="winds-t143-platform-baseline-v1")
    parser.add_argument(
        "--purpose",
        default="Diagnostic same-host minimal-renderer baseline; never substitutes for the frozen product RSS gate.",
    )
    args = parser.parse_args()

    binary = str(Path(args.binary).resolve())
    if not Path(binary).is_file():
        raise SystemExit(f"T143 platform baseline binary missing: {binary}")

    home = tempfile.mkdtemp(prefix="winds-t143-platform-baseline-")
    proc: subprocess.Popen[bytes] | None = None
    log: BinaryIO | None = None
    try:
        proc, _, log = native.start(binary, home)
        await_ready(proc, log, args.ready_window_pattern)
        idle = native.idle_campaign(proc, args.idle_seconds, settle=2.0)
    finally:
        if proc is not None:
            native.terminate(proc)
        if log is not None:
            log.close()
        shutil.rmtree(home, ignore_errors=True)

    checks = {
        "renderer_present_every_idle_sample": idle["renderer_present_every_sample"],
        "renderer_host_process_count_ge_2_every_idle_sample": idle["min_process_count"] >= 2,
    }
    result = {
        "schema": args.schema,
        "purpose": args.purpose,
        "binary": binary,
        "idle": idle,
        "checks": checks,
        "diagnostic_renderer_host_idle_rss_le_300_mib": idle["rss_max_bytes"] <= 300 * 1024 * 1024,
    }
    Path(args.output).write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    if not all(checks.values()):
        raise SystemExit(f"T143 platform baseline integrity failed: {[key for key, value in checks.items() if not value]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
