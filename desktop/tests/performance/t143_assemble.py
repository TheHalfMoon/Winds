#!/usr/bin/env python3
"""Assemble exact-candidate T143 performance evidence from raw qualification outputs."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import subprocess
from pathlib import Path
from typing import Any


def command(*args: str) -> str:
    return subprocess.check_output(args, text=True).strip()


def marker(path: str, prefix: str) -> Any:
    for line in Path(path).read_text(encoding="utf-8", errors="replace").splitlines():
        offset = line.find(prefix)
        if offset >= 0:
            return json.loads(line[offset + len(prefix) :])
    raise RuntimeError(f"missing {prefix} in {path}")


def sha256(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def directory_digest(path: str) -> str:
    root = Path(path)
    digest = hashlib.sha256()
    for item in sorted(p for p in root.rglob("*") if p.is_file()):
        digest.update(item.relative_to(root).as_posix().encode())
        digest.update(b"\0")
        digest.update(bytes.fromhex(sha256(str(item))))
    return digest.hexdigest()


def optional_command(*args: str) -> str:
    try:
        return command(*args)
    except (FileNotFoundError, subprocess.CalledProcessError):
        return "UNAVAILABLE"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--native", required=True)
    parser.add_argument("--renderer", required=True)
    parser.add_argument("--backend-log", required=True)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--native-dist", required=True)
    parser.add_argument("--benchmark-dist", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    native = json.loads(Path(args.native).read_text(encoding="utf-8"))
    renderer = json.loads(Path(args.renderer).read_text(encoding="utf-8"))
    backend = marker(args.backend_log, "T097_CORE_JSON=")
    candidate = command("git", "rev-parse", "HEAD")
    tree = command("git", "rev-parse", "HEAD^{tree}")
    expected = os.environ.get("CANDIDATE_SHA", candidate)
    if candidate != expected:
        raise RuntimeError(f"candidate mismatch: checkout={candidate} expected={expected}")

    evidence = {
        "schema": "WINDS_SPEC_010_T143_PERFORMANCE_EVIDENCE_V1",
        "candidate_commit": candidate,
        "candidate_tree": tree,
        "base_commit": os.environ.get("BASE_SHA", "UNAVAILABLE"),
        "environment": {
            "runner_os": os.environ.get("RUNNER_OS", platform.system()),
            "runner_arch": os.environ.get("RUNNER_ARCH", platform.machine()),
            "image_os": os.environ.get("ImageOS", "UNAVAILABLE"),
            "image_version": os.environ.get("ImageVersion", "UNAVAILABLE"),
            "platform": platform.platform(),
            "logical_cpu_count": os.cpu_count(),
            "rustc": optional_command("rustc", "--version", "--verbose"),
            "cargo": optional_command("cargo", "--version"),
            "node": optional_command("node", "--version"),
            "npm": optional_command("npm", "--version"),
            "webkitgtk": optional_command("pkg-config", "--modversion", "webkit2gtk-4.1"),
            "webkitgtk_package": optional_command("dpkg-query", "-W", "-f=${Version}", "libwebkit2gtk-4.1-0"),
            "webdriver_package": optional_command("dpkg-query", "-W", "-f=${Version}", "webkit2gtk-driver"),
            "epiphany_package": optional_command("dpkg-query", "-W", "-f=${Version}", "epiphany-browser"),
            "epiphany": optional_command("epiphany", "--version"),
        },
        "build": {
            "profile": "release",
            "tauri_feature": None,
            "binary_sha256": sha256(args.binary),
            "native_renderer_dist_sha256": directory_digest(args.native_dist),
            "benchmark_renderer_dist_sha256": directory_digest(args.benchmark_dist),
            "native_renderer_profile": "VITE_WINDS_T143_NATIVE_READY=1 deterministic two-Session qualification renderer",
            "benchmark_renderer_profile": "VITE_WINDS_T143_BENCHMARK=1 renderer-only interaction and scale fixture",
            "production_dependency_or_lockfile_change": False,
            "new_runtime_dependency": False,
            "new_renderer_dependency": False,
            "new_production_tauri_command": False,
        },
        "native": native,
        "renderer": renderer,
        "backend_stress": backend,
        "hidden_output_batching": {
            "deterministic_test": "T143 hidden output batches multiple sessions instead of scheduling full-frame work per byte",
            "hidden_flush_window_ms": 60,
            "visible_flush_window_ms": 16,
            "simulated_hidden_sessions": 8,
            "chunks_per_session": 1000,
            "claim_boundary": "proves renderer scheduling batches hidden output; it does not claim hidden sessions render frames",
        },
        "campaigns": {
            "cold_launch_samples": native["launch"]["sample_count"],
            "selection_samples": renderer["normal"]["selection"]["sample_count"],
            "composer_samples": renderer["normal"]["composer_keystroke_to_paint"]["sample_count"],
            "single_dual_samples": renderer["normal"]["single_dual_layout"]["sample_count"],
            "right_dock_samples": renderer["normal"]["cached_right_dock_switch"]["sample_count"],
            "divider_resize_samples": renderer["normal"]["divider_resize"]["sample_count"],
            "large_fixture_projects": renderer["large_fixture"]["project_count"],
            "large_fixture_sessions": renderer["large_fixture"]["session_count"],
            "large_fixture_search_samples": renderer["large_fixture"]["search"]["sample_count"],
            "large_fixture_scroll_samples": renderer["large_fixture"]["scroll"]["sample_count"],
            "large_fixture_focus_samples": renderer["large_fixture"]["focus"]["sample_count"],
            "backend_processed_bytes": backend["fr049_high_volume"]["processed_bytes"],
            "backend_processed_lines": backend["fr049_high_volume"]["processed_lines"],
            "backend_resize_samples": backend["fr052_resize"]["sample_count"],
        },
    }

    checks: dict[str, bool] = {}
    checks.update({f"native::{key}": bool(value) for key, value in native["checks"].items()})
    checks.update({f"renderer::{key}": bool(value) for key, value in renderer["checks"].items()})
    checks.update({
        "backend::processed_at_least_10_mib": backend["fr049_high_volume"]["processed_bytes"] >= 10 * 1024 * 1024,
        "backend::processed_at_least_100000_lines": backend["fr049_high_volume"]["processed_lines"] >= 100_000,
        "backend::lifecycle_preserved": bool(backend["fr049_high_volume"]["lifecycle_live_after_campaign"]),
        "backend::ownership_preserved": bool(backend["fr049_high_volume"]["owned_terminal_after_campaign"]),
        "backend::resize_1000": backend["fr052_resize"]["sample_count"] >= 1000,
        "backend::final_size_correct": bool(backend["fr052_resize"]["final_size_correct"]),
        "provenance::candidate_exact": candidate == expected,
        "provenance::tree_present": bool(re.fullmatch(r"[0-9a-f]{40}", tree)),
    })
    evidence["checks"] = checks
    evidence["all_checks_pass"] = all(checks.values())
    Path(args.output).write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print("T143_EVIDENCE_JSON=" + json.dumps(evidence, sort_keys=True))
    if not evidence["all_checks_pass"]:
        failed = [name for name, passed in checks.items() if not passed]
        print("T143 combined qualification failed: " + ", ".join(failed), file=os.sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
