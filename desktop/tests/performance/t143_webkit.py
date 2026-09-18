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


WEBDRIVER_SCRIPT_TIMEOUT_MS = 600_000
WEBDRIVER_CAMPAIGN_HTTP_TIMEOUT_SECONDS = 660.0


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


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def renderer_preflight(base: str, session: str) -> dict[str, Any]:
    value = execute(
        base,
        session,
        r"""
return {
  href: location.href,
  readyState: document.readyState,
  windsApp: Boolean(document.querySelector('.winds-app')),
  chatSelector: Boolean(document.querySelector('#chat-session-target')),
  chatOptionCount: document.querySelector('#chat-session-target')?.querySelectorAll('option').length ?? 0,
  chatOptionValues: Array.from(document.querySelectorAll('#chat-session-target option')).map((option) => option.value),
  chatComposerCount: document.querySelectorAll('.chat-composer textarea').length,
  dualMode: document.querySelector('.dual-session-layout')?.dataset.mode ?? null,
  sessionSlotCount: document.querySelectorAll('.session-slot').length,
  activityButtonCount: Array.from(document.querySelectorAll('.session-slot button')).filter((button) => button.textContent?.trim() === 'Activity').length,
  rightDockPresent: Boolean(document.querySelector('.right-dock')),
  benchmarkPhase: document.documentElement.dataset.t143Phase ?? null,
  userAgent: navigator.userAgent,
  viewport: { width: innerWidth, height: innerHeight, devicePixelRatio },
};
""",
    )
    if not isinstance(value, dict):
        raise RuntimeError(f"renderer preflight returned non-object: {value!r}")
    return value


def best_effort_browser_phase(base: str, session: str | None) -> str | None:
    if not session:
        return None
    try:
        phase = execute(base, session, "return document.documentElement.dataset.t143Phase ?? null;")
        return phase if isinstance(phase, str) else None
    except Exception:
        return None


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
    document.documentElement.dataset.t143Sample = String(index);
    await frame();
    await phase(index);
    const started = performance.now();
    action();
    await Promise.resolve();
    if (predicate) await waitFor(predicate, 'committed UI state');
    const painted = await frame();
    return painted - started;
  };

  document.documentElement.dataset.t143Phase = 'normal:selector-precondition';
  await waitFor(() => document.querySelector('#chat-session-target')?.querySelectorAll('option').length >= 3, 'Chat Session selector');
  const sessionA = 'fixture-winds\u0000fixture-session-a';
  const sessionB = 'fixture-winds\u0000fixture-session-b';
  const selectExactSession = (target) => {
    const currentSelector = document.querySelector('#chat-session-target');
    if (!currentSelector) throw new Error('Chat Session selector missing during selection sample');
    currentSelector.value = target;
    currentSelector.dispatchEvent(new Event('change', { bubbles: true }));
  };
  const exactSessionCommitted = (target, sessionId) => {
    const currentSelector = document.querySelector('#chat-session-target');
    const identity = document.querySelector('.chat-session-identity');
    const focusedIdentity = document.querySelector('.session-slot[data-focused="true"] .session-identity');
    return currentSelector?.value === target
      && identity?.textContent?.includes(sessionId)
      && focusedIdentity?.textContent?.includes(sessionId);
  };
  document.documentElement.dataset.t143Phase = 'normal:selection';
  const selection = [];
  for (let index = 0; index < 1000; index += 1) {
    const target = index % 2 === 0 ? sessionB : sessionA;
    const sessionId = index % 2 === 0 ? 'fixture-session-b' : 'fixture-session-a';
    selection.push(await paintSample(
      index,
      () => selectExactSession(target),
      () => exactSessionCommitted(target, sessionId),
    ));
  }

  document.documentElement.dataset.t143Phase = 'normal:composer';
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

  document.documentElement.dataset.t143Phase = 'normal:activity-precondition';
  // Put both visible Workbench Slots on representative Activity output before focus/layout stress.
  const activityButtons = Array.from(document.querySelectorAll('.session-slot button')).filter((button) => button.textContent?.trim() === 'Activity');
  for (const button of activityButtons) button.click();
  await waitFor(() => document.querySelectorAll('.session-slot .session-work-event:not([hidden])').length >= 2, 'two visible Session work events');

  document.documentElement.dataset.t143Phase = 'normal:layout';
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

  document.documentElement.dataset.t143Phase = 'normal:right-dock';
  // Ensure an exact committed target is selected so the right dock remains bound.
  selectExactSession(sessionA);
  await waitFor(
    () => exactSessionCommitted(sessionA, 'fixture-session-a') && Boolean(document.querySelector('.right-dock')),
    'right dock exact Session binding',
  );
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

  document.documentElement.dataset.t143Phase = 'normal:resize';
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

  document.documentElement.dataset.t143Phase = 'normal:complete';
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
})().then((value) => done({ ok: true, value })).catch((error) => done({
  ok: false,
  phase: document.documentElement.dataset.t143Phase ?? null,
  sampleIndex: document.documentElement.dataset.t143Sample ?? null,
  errorName: error?.name ?? null,
  errorMessage: error?.message ?? String(error),
  errorStack: error?.stack ?? null,
  error: String(error?.message ?? error?.stack ?? error),
}));
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
    document.documentElement.dataset.t143Sample = String(index);
    await frame();
    await phase(index);
    const started = performance.now();
    action();
    await Promise.resolve();
    if (predicate) await waitFor(predicate, 'large-fixture committed state');
    const painted = await frame();
    return painted - started;
  };

  document.documentElement.dataset.t143Phase = 'large:fixture-precondition';
  await waitFor(() => document.querySelectorAll('.project-group').length >= 101, '101 Projects');
  await waitFor(() => document.querySelectorAll('.session-row').length >= 1003, '1003 Sessions');
  const projectCount = document.querySelectorAll('.project-group').length;
  const sessionCount = document.querySelectorAll('.session-row').length;
  const search = document.querySelector('input[aria-label="Search Projects and Sessions"]');
  const list = document.querySelector('.project-list');
  if (!search || !list) throw new Error('large fixture search/list missing');

  const inputValueSetter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
  if (!inputValueSetter) throw new Error('native HTMLInputElement value setter unavailable');
  const setSearchValue = (value) => {
    inputValueSetter.call(search, value);
    search.dispatchEvent(new Event('input', { bubbles: true }));
  };
  const visibleSessionRows = () => Array.from(document.querySelectorAll('.session-row'))
    .filter((row) => !row.closest('[hidden], [inert], [data-search-hidden="true"]'));

  const base = document.querySelector('.session-row[data-session-id="fixture-session-a"]');
  base?.click();
  // Selecting from Projects intentionally returns presentation focus to Chat; reopen Projects via the activity rail.
  const projectsRail = document.querySelector('button[aria-label="Open Projects tool window"]');
  projectsRail?.click();
  await waitFor(() => document.querySelector('.left-dock'), 'Projects tool window after exact selection');

  document.documentElement.dataset.t143Phase = 'large:search';
  const searchSamples = [];
  for (let index = 0; index < 200; index += 1) {
    const query = index % 2 === 0 ? `t143-project-${String(index % 100).padStart(3, '0')}-session-09` : '';
    searchSamples.push(await sample(index, () => {
      setSearchValue(query);
    }, () => query ? visibleSessionRows().length === 1 : visibleSessionRows().length >= 1003));
  }
  if (search.value !== '') {
    setSearchValue('');
    await waitFor(() => visibleSessionRows().length >= 1003, 'large fixture restored after search');
  }

  document.documentElement.dataset.t143Phase = 'large:scroll';
  const scrollSamples = [];
  for (let index = 0; index < 1000; index += 1) {
    await frame();
    const started = performance.now();
    const maxScroll = Math.max(1, list.scrollHeight - list.clientHeight);
    list.scrollTop = (index * 97) % maxScroll;
    const painted = await frame();
    scrollSamples.push(painted - started);
  }

  document.documentElement.dataset.t143Phase = 'large:focus';
  const focusRows = visibleSessionRows();
  const focusSamples = [];
  for (let index = 0; index < 1000; index += 1) {
    const row = focusRows[(index * 37) % focusRows.length];
    focusSamples.push(await sample(index, () => row.focus(), () => document.activeElement === row));
  }

  document.documentElement.dataset.t143Phase = 'large:complete';
  return {
    userAgent: navigator.userAgent,
    projectCount,
    sessionCount,
    searchSamples,
    scrollSamples,
    focusSamples,
    selectedBaseSessionPreserved: Boolean(document.querySelector('.session-row-shell[data-selected="true"] .session-row[data-session-id="fixture-session-a"]')),
  };
})().then((value) => done({ ok: true, value })).catch((error) => done({
  ok: false,
  phase: document.documentElement.dataset.t143Phase ?? null,
  sampleIndex: document.documentElement.dataset.t143Sample ?? null,
  errorName: error?.name ?? null,
  errorMessage: error?.message ?? String(error),
  errorStack: error?.stack ?? null,
  error: String(error?.message ?? error?.stack ?? error),
}));
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-url", default="http://127.0.0.1:4173/")
    parser.add_argument("--webdriver", default="http://127.0.0.1:4444")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    output_path = Path(args.output)
    preflight_path = output_path.with_name("renderer-preflight.json")
    failure_path = output_path.with_name("renderer-failure.json")
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

    session: str | None = None
    resolved_capabilities: dict[str, Any] = {}
    phase = "session:create"
    try:
        session_value = request_json(args.webdriver, "POST", "/session", capabilities)
        if not isinstance(session_value, dict) or not session_value.get("sessionId"):
            raise RuntimeError(f"WebDriver did not return a session id: {session_value!r}")
        session = session_value["sessionId"]
        resolved_capabilities = session_value.get("capabilities", {})

        phase = "session:timeouts"
        request_json(args.webdriver, "POST", f"/session/{session}/timeouts", {"script": WEBDRIVER_SCRIPT_TIMEOUT_MS, "pageLoad": 30_000})

        phase = "normal:navigate"
        normal_url = f"{args.base_url}?tool=chat&theme=dark"
        request_json(args.webdriver, "POST", f"/session/{session}/url", {"url": normal_url})
        phase = "normal:shell-ready"
        wait_for_shell(args.webdriver, session)
        phase = "normal:preflight"
        preflight = renderer_preflight(args.webdriver, session)
        write_json(preflight_path, {
            "schema": "winds-t143-renderer-preflight-v1",
            "browser_name": browser_name,
            "browser_binary": binary,
            "browser_argument": browser_arg,
            "webdriver_capabilities": resolved_capabilities,
            "normal": preflight,
        })

        phase = "normal:campaign"
        normal = execute(
            args.webdriver,
            session,
            NORMAL_CAMPAIGN,
            async_=True,
            timeout=WEBDRIVER_CAMPAIGN_HTTP_TIMEOUT_SECONDS,
        )
        if not isinstance(normal, dict) or not normal.get("ok"):
            detail = normal if isinstance(normal, dict) else {"value": normal}
            raise RuntimeError(f"normal renderer campaign failed: {detail}")

        phase = "large:navigate"
        large_url = f"{args.base_url}?tool=projects&theme=dark&perf=t143-large"
        request_json(args.webdriver, "POST", f"/session/{session}/url", {"url": large_url})
        phase = "large:shell-ready"
        wait_for_shell(args.webdriver, session)
        phase = "large:preflight"
        large_preflight = renderer_preflight(args.webdriver, session)
        prior_preflight = json.loads(preflight_path.read_text(encoding="utf-8"))
        prior_preflight["large"] = large_preflight
        write_json(preflight_path, prior_preflight)

        phase = "large:campaign"
        large = execute(
            args.webdriver,
            session,
            LARGE_CAMPAIGN,
            async_=True,
            timeout=WEBDRIVER_CAMPAIGN_HTTP_TIMEOUT_SECONDS,
        )
        if not isinstance(large, dict) or not large.get("ok"):
            detail = large if isinstance(large, dict) else {"value": large}
            raise RuntimeError(f"large renderer campaign failed: {detail}")
    except Exception as error:
        write_json(failure_path, {
            "schema": "winds-t143-renderer-failure-v1",
            "python_phase": phase,
            "browser_phase": best_effort_browser_phase(args.webdriver, session),
            "error_type": type(error).__name__,
            "error": str(error),
            "browser_name": browser_name,
            "browser_binary": binary,
            "browser_argument": browser_arg,
            "webdriver_capabilities": resolved_capabilities,
            "preflight_path": str(preflight_path),
        })
        raise
    finally:
        if session:
            try:
                request_json(args.webdriver, "DELETE", f"/session/{session}")
            except Exception as error:
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
    write_json(output_path, result)
    failure_path.unlink(missing_ok=True)
    print(f"T143_RENDERER_JSON={json.dumps(result, sort_keys=True)}")
    failed = [name for name, passed in checks.items() if not passed]
    if failed:
        print(f"T143 renderer qualification failed: {failed}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
