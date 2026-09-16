#!/usr/bin/env python3
"""T143 WebKitGTK renderer interaction qualification using only W3C WebDriver HTTP."""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


WEBDRIVER_SCRIPT_TIMEOUT_MS = 300_000
WEBDRIVER_CAMPAIGN_HTTP_TIMEOUT_SECONDS = 330.0


def request_json(
    base: str,
    method: str,
    path: str,
    payload: dict[str, Any] | None = None,
    timeout: float = 30.0,
) -> Any:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        f"{base}{path}",
        data=data,
        method=method,
        headers={"Content-Type": "application/json; charset=utf-8"},
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as error:
        body = error.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"WebDriver {method} {path} failed: HTTP {error.code}: {body}") from error
    if isinstance(body, dict) and isinstance(body.get("value"), dict) and body["value"].get("error"):
        raise RuntimeError(f"WebDriver {method} {path} failed: {body['value']}")
    return body.get("value") if isinstance(body, dict) else body


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


def execute(
    base: str,
    session: str,
    script: str,
    async_: bool = False,
    timeout: float = 30.0,
) -> Any:
    endpoint = "async" if async_ else "sync"
    return request_json(
        base,
        "POST",
        f"/session/{session}/execute/{endpoint}",
        {"script": script, "args": []},
        timeout=timeout,
    )


def wait_for_shell(base: str, session: str) -> None:
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        ready = execute(
            base,
            session,
            "return document.readyState === 'complete' && Boolean(document.querySelector('.winds-app'));",
        )
        if ready:
            return
        time.sleep(0.05)
    raise RuntimeError("Winds fixture shell did not become ready in WebKitGTK")


NORMAL_CAMPAIGN = r"""
const done = arguments[arguments.length - 1];
(async () => {
  const frame = () => new Promise((resolve) => requestAnimationFrame(resolve));
  const phase = (index) => new Promise((resolve) => setTimeout(resolve, 1 + (index % 15)));
  const waitFor = async (predicate, label, timeoutMs = 2000) => {
    const deadline = performance.now() + timeoutMs;
    while (performance.now() < deadline) {
      if (predicate()) return;
      await frame();
    }
    throw new Error(`timed out waiting for ${label}`);
  };
  const paintSample = async (index, action, predicate) => {
    await frame();
    await phase(index);
    const started = performance.now();
    action();
    await Promise.resolve();
    if (predicate) await waitFor(predicate, 'committed UI state');
    const painted = await frame();
    return painted - started;
  };

  await waitFor(() => document.querySelector('#chat-session-target')?.querySelectorAll('option').length >= 3, 'Chat Session selector');
  const selector = document.querySelector('#chat-session-target');
  const sessionA = 'fixture-winds\u0000fixture-session-a';
  const sessionB = 'fixture-winds\u0000fixture-session-b';
  const selection = [];
  for (let index = 0; index < 1000; index += 1) {
    const target = index % 2 === 0 ? sessionB : sessionA;
    selection.push(await paintSample(index, () => {
      selector.value = target;
      selector.dispatchEvent(new Event('change', { bubbles: true }));
    }, () => selector.value === target));
  }

  const composer = document.querySelector('.chat-composer textarea');
  if (!composer) throw new Error('Chat composer textarea missing');
  const wasDisabled = composer.disabled;
  composer.disabled = false;
  composer.focus();
  const composerPaint = [];
  for (let index = 0; index < 1000; index += 1) {
    composerPaint.push(await paintSample(index, () => {
      composer.value = `t143-${index}`;
      composer.dispatchEvent(new Event('input', { bubbles: true }));
    }, null));
  }
  composer.value = '';
  composer.disabled = wasDisabled;

  // Put both visible Workbench Slots on representative Activity output before focus/layout stress.
  const activityButtons = Array.from(document.querySelectorAll('.session-slot button')).filter((button) => button.textContent?.trim() === 'Activity');
  for (const button of activityButtons) button.click();
  await waitFor(() => document.querySelectorAll('.session-slot .session-work-event:not([hidden])').length >= 2, 'two visible Session work events');

  const layoutSamples = [];
  for (let cycle = 0; cycle < 500; cycle += 1) {
    const rightClose = Array.from(document.querySelectorAll('.session-slot[data-slot="right"] .session-slot-controls button')).find((button) => button.textContent?.trim() === 'Close');
    if (!rightClose) throw new Error('right Slot Close control missing');
    layoutSamples.push(await paintSample(cycle * 2, () => rightClose.click(), () => document.querySelector('.dual-session-layout')?.dataset.mode === 'SINGLE'));
    const open = Array.from(document.querySelectorAll('.dual-session-actions button')).find((button) => button.textContent?.trim() === 'Open second Session');
    if (!open) throw new Error('Open second Session control missing');
    layoutSamples.push(await paintSample(cycle * 2 + 1, () => open.click(), () => document.querySelector('.dual-session-layout')?.dataset.mode === 'DUAL'));
    // Restore representative Activity output after each remount.
    for (const button of Array.from(document.querySelectorAll('.session-slot button')).filter((candidate) => candidate.textContent?.trim() === 'Activity')) button.click();
  }

  // Ensure an exact target is selected so the right dock remains bound.
  selector.value = sessionA;
  selector.dispatchEvent(new Event('change', { bubbles: true }));
  await waitFor(() => document.querySelector('.right-dock'), 'right dock');
  const rightDockSamples = [];
  const tabs = () => Array.from(document.querySelectorAll('.right-tab'));
  for (let index = 0; index < 1000; index += 1) {
    const desired = index % 2 === 0 ? 'Changes' : 'Files';
    const tab = tabs().find((candidate) => candidate.textContent?.trim() === desired);
    if (!tab) throw new Error(`right dock ${desired} tab missing`);
    rightDockSamples.push(await paintSample(index, () => tab.click(), () => {
      const status = document.querySelector('.right-dock [role="status"]')?.textContent ?? '';
      return tab.getAttribute('aria-selected') === 'true' && !/Binding|Loading/.test(status);
    }));
  }

  const divider = document.querySelector('input[aria-label="Resize Session divider"]');
  const resizeSamples = [];
  if (!divider) throw new Error('Session divider missing');
  for (let index = 0; index < 1000; index += 1) {
    const value = String(3000 + ((index % 40) * 100));
    resizeSamples.push(await paintSample(index, () => {
      divider.value = value;
      divider.dispatchEvent(new Event('change', { bubbles: true }));
    }, () => divider.value === value));
  }

  return {
    userAgent: navigator.userAgent,
    viewport: { width: innerWidth, height: innerHeight, devicePixelRatio },
    visibleSessionWorkEvents: document.querySelectorAll('.session-slot .session-work-event:not([hidden])').length,
    selection,
    composerPaint,
    layoutSamples,
    rightDockSamples,
    resizeSamples,
    composerMeasurementBoundary: 'measurement-only DOM enablement; no submit or runtime dispatch; canonical T139 composer availability remains unchanged',
  };
})().then((value) => done({ ok: true, value })).catch((error) => done({ ok: false, error: String(error?.stack ?? error) }));
"""

LARGE_CAMPAIGN = r"""
const done = arguments[arguments.length - 1];
(async () => {
  const frame = () => new Promise((resolve) => requestAnimationFrame(resolve));
  const phase = (index) => new Promise((resolve) => setTimeout(resolve, 1 + (index % 15)));
  const waitFor = async (predicate, label, timeoutMs = 15000) => {
    const deadline = performance.now() + timeoutMs;
    while (performance.now() < deadline) {
      if (predicate()) return;
      await frame();
    }
    throw new Error(`timed out waiting for ${label}`);
  };
  const sample = async (index, action, predicate) => {
    await frame();
    await phase(index);
    const started = performance.now();
    action();
    await Promise.resolve();
    if (predicate) await waitFor(predicate, 'large-fixture committed state');
    const painted = await frame();
    return painted - started;
  };

  await waitFor(() => document.querySelectorAll('.project-group').length >= 101, '101 Projects');
  await waitFor(() => document.querySelectorAll('.session-row').length >= 1003, '1003 Sessions');
  const projectCount = document.querySelectorAll('.project-group').length;
  const sessionCount = document.querySelectorAll('.session-row').length;
  const search = document.querySelector('input[aria-label="Search Projects and Sessions"]');
  const list = document.querySelector('.project-list');
  if (!search || !list) throw new Error('large fixture search/list missing');

  const base = document.querySelector('.session-row[data-session-id="fixture-session-a"]');
  base?.click();
  // Selecting from Projects intentionally returns presentation focus to Chat; reopen Projects via the activity rail.
  const projectsRail = document.querySelector('button[aria-label="Open Projects tool window"]');
  projectsRail?.click();
  await waitFor(() => document.querySelector('.left-dock'), 'Projects tool window after exact selection');

  const searchSamples = [];
  for (let index = 0; index < 200; index += 1) {
    const query = index % 2 === 0 ? `t143-project-${String(index % 100).padStart(3, '0')}-session-09` : '';
    searchSamples.push(await sample(index, () => {
      search.value = query;
      search.dispatchEvent(new Event('input', { bubbles: true }));
    }, () => query ? document.querySelectorAll('.session-row').length === 1 : document.querySelectorAll('.session-row').length >= 1003));
  }
  if (search.value !== '') {
    search.value = '';
    search.dispatchEvent(new Event('input', { bubbles: true }));
    await waitFor(() => document.querySelectorAll('.session-row').length >= 1003, 'large fixture restored after search');
  }

  const scrollSamples = [];
  for (let index = 0; index < 1000; index += 1) {
    await frame();
    const started = performance.now();
    const maxScroll = Math.max(1, list.scrollHeight - list.clientHeight);
    list.scrollTop = (index * 97) % maxScroll;
    const painted = await frame();
    scrollSamples.push(painted - started);
  }

  const focusRows = Array.from(document.querySelectorAll('.session-row'));
  const focusSamples = [];
  for (let index = 0; index < 1000; index += 1) {
    const row = focusRows[(index * 37) % focusRows.length];
    focusSamples.push(await sample(index, () => row.focus(), () => document.activeElement === row));
  }

  return {
    userAgent: navigator.userAgent,
    projectCount,
    sessionCount,
    searchSamples,
    scrollSamples,
    focusSamples,
    selectedBaseSessionPreserved: Boolean(document.querySelector('.session-row-shell[data-selected="true"] .session-row[data-session-id="fixture-session-a"]')),
  };
})().then((value) => done({ ok: true, value })).catch((error) => done({ ok: false, error: String(error?.stack ?? error) }));
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-url", default="http://127.0.0.1:4173/")
    parser.add_argument("--webdriver", default="http://127.0.0.1:4444")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    binary = os.environ.get("WINDS_T143_WEBKIT_BINARY", "/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/MiniBrowser")
    browser_name = os.environ.get("WINDS_T143_WEBKIT_BROWSER_NAME", "MiniBrowser")
    browser_arg = os.environ.get("WINDS_T143_WEBKIT_ARGUMENT", "--automation")
    capabilities = {
        "capabilities": {
            "alwaysMatch": {
                "browserName": browser_name,
                "webkitgtk:browserOptions": {"binary": binary, "args": [browser_arg]},
            }
        }
    }
    session_value = request_json(args.webdriver, "POST", "/session", capabilities)
    if not isinstance(session_value, dict) or not session_value.get("sessionId"):
        raise RuntimeError(f"WebDriver did not return a session id: {session_value!r}")
    session = session_value["sessionId"]
    resolved_capabilities = session_value.get("capabilities", {})
    try:
        request_json(args.webdriver, "POST", f"/session/{session}/timeouts", {"script": WEBDRIVER_SCRIPT_TIMEOUT_MS, "pageLoad": 30_000})
        normal_url = f"{args.base_url}?tool=chat&theme=dark"
        request_json(args.webdriver, "POST", f"/session/{session}/url", {"url": normal_url})
        wait_for_shell(args.webdriver, session)
        normal = execute(
            args.webdriver,
            session,
            NORMAL_CAMPAIGN,
            async_=True,
            timeout=WEBDRIVER_CAMPAIGN_HTTP_TIMEOUT_SECONDS,
        )
        if not isinstance(normal, dict) or not normal.get("ok"):
            raise RuntimeError(f"normal renderer campaign failed: {normal}")

        large_url = f"{args.base_url}?tool=projects&theme=dark&perf=t143-large"
        request_json(args.webdriver, "POST", f"/session/{session}/url", {"url": large_url})
        wait_for_shell(args.webdriver, session)
        large = execute(
            args.webdriver,
            session,
            LARGE_CAMPAIGN,
            async_=True,
            timeout=WEBDRIVER_CAMPAIGN_HTTP_TIMEOUT_SECONDS,
        )
        if not isinstance(large, dict) or not large.get("ok"):
            raise RuntimeError(f"large renderer campaign failed: {large}")
    finally:
        try:
            request_json(args.webdriver, "DELETE", f"/session/{session}")
        except Exception as error:  # best-effort cleanup after evidence is already captured
            print(f"WebDriver cleanup warning: {error}", file=sys.stderr)

    normal_value = normal["value"]
    large_value = large["value"]
    result = {
        "schema": "winds-t143-renderer-performance-v1",
        "browser_name": browser_name,
        "browser_binary": binary,
        "browser_argument": browser_arg,
        "webdriver_capabilities": resolved_capabilities,
        "normal": {
            "user_agent": normal_value["userAgent"],
            "viewport": normal_value["viewport"],
            "visible_session_work_events": normal_value["visibleSessionWorkEvents"],
            "selection": summary(normal_value["selection"]),
            "composer_keystroke_to_paint": summary(normal_value["composerPaint"]),
            "single_dual_layout": summary(normal_value["layoutSamples"]),
            "cached_right_dock_switch": summary(normal_value["rightDockSamples"]),
            "divider_resize": summary(normal_value["resizeSamples"]),
            "composer_measurement_boundary": normal_value["composerMeasurementBoundary"],
        },
        "large_fixture": {
            "project_count": large_value["projectCount"],
            "session_count": large_value["sessionCount"],
            "user_agent": large_value["userAgent"],
            "search": summary(large_value["searchSamples"]),
            "scroll": summary(large_value["scrollSamples"]),
            "focus": summary(large_value["focusSamples"]),
            "selected_base_session_preserved": large_value["selectedBaseSessionPreserved"],
        },
    }

    checks = {
        "selection_p95_le_50_ms": result["normal"]["selection"]["p95_ms"] <= 50.0,
        "single_dual_p95_le_100_ms": result["normal"]["single_dual_layout"]["p95_ms"] <= 100.0,
        "composer_p95_le_16_ms": result["normal"]["composer_keystroke_to_paint"]["p95_ms"] <= 16.0,
        "cached_right_dock_p95_le_50_ms": result["normal"]["cached_right_dock_switch"]["p95_ms"] <= 50.0,
        "large_fixture_projects_ge_100": result["large_fixture"]["project_count"] >= 100,
        "large_fixture_sessions_ge_1000": result["large_fixture"]["session_count"] >= 1000,
        "two_visible_session_work_events": result["normal"]["visible_session_work_events"] >= 2,
        "resize_p95_le_100_ms": result["normal"]["divider_resize"]["p95_ms"] <= 100.0,
        "large_search_p95_le_50_ms": result["large_fixture"]["search"]["p95_ms"] <= 50.0,
        "large_scroll_p95_le_50_ms": result["large_fixture"]["scroll"]["p95_ms"] <= 50.0,
        "large_focus_p95_le_50_ms": result["large_fixture"]["focus"]["p95_ms"] <= 50.0,
        "large_fixture_selection_stable": result["large_fixture"]["selected_base_session_preserved"],
    }
    result["checks"] = checks
    Path(args.output).write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"T143_RENDERER_JSON={json.dumps(result, sort_keys=True)}")
    failed = [name for name, passed in checks.items() if not passed]
    if failed:
        print(f"T143 renderer qualification failed: {failed}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
