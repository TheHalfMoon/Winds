#!/usr/bin/env python3
"""Diagnostic-only T143 WebKitGTK Session-selection probe.

This probe never contributes to qualification checks. It exercises a small number
of exact Chat Session selections in a separate WebDriver session and persists
DOM/binding state so a subsequent authoritative renderer-campaign failure can be
attributed without changing its sample floors or thresholds.
"""
from __future__ import annotations

import argparse
import json
import os
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any


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


def execute(base: str, session: str, script: str, timeout: float = 30.0) -> Any:
    return request_json(
        base,
        "POST",
        f"/session/{session}/execute/sync",
        {"script": script, "args": []},
        timeout=timeout,
    )


def wait_until(base: str, session: str, script: str, label: str, timeout_seconds: float = 15.0) -> None:
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if execute(base, session, script):
            return
        time.sleep(0.05)
    raise RuntimeError(f"timed out waiting for {label}")


SNAPSHOT_SCRIPT = r"""
const selector = document.querySelector('#chat-session-target');
const selected = selector?.selectedOptions?.[0] ?? null;
const identity = document.querySelector('.chat-session-identity');
return {
  href: location.href,
  readyState: document.readyState,
  phase: document.documentElement.dataset.t143Phase ?? null,
  selectorPresent: Boolean(selector),
  selectorValue: selector?.value ?? null,
  selectorValueCodePoints: selector ? Array.from(selector.value).map((char) => char.codePointAt(0)) : [],
  selectedIndex: selector?.selectedIndex ?? -1,
  selectedOptionValue: selected?.value ?? null,
  selectedOptionText: selected?.textContent ?? null,
  optionValues: selector ? Array.from(selector.options).map((option) => option.value) : [],
  optionValueCodePoints: selector ? Array.from(selector.options).map((option) => Array.from(option.value).map((char) => char.codePointAt(0))) : [],
  optionTexts: selector ? Array.from(selector.options).map((option) => option.textContent ?? '') : [],
  chatIdentityText: identity?.textContent ?? null,
  dualMode: document.querySelector('.dual-session-layout')?.dataset.mode ?? null,
  sessionSlotCount: document.querySelectorAll('.session-slot').length,
  rightDockPresent: Boolean(document.querySelector('.right-dock')),
  rightDockText: document.querySelector('.right-dock')?.textContent?.slice(0, 500) ?? null,
};
"""


def selection_attempt_script(target: str) -> str:
    encoded = json.dumps(target)
    return f"""
const selector = document.querySelector('#chat-session-target');
if (!selector) return {{ ok: false, reason: 'selector-missing' }};
const target = {encoded};
const optionValues = Array.from(selector.options).map((option) => option.value);
selector.value = target;
const afterAssignment = selector.value;
selector.dispatchEvent(new Event('change', {{ bubbles: true }}));
return {{
  ok: true,
  target,
  targetCodePoints: Array.from(target).map((char) => char.codePointAt(0)),
  optionValues,
  optionValueCodePoints: optionValues.map((value) => Array.from(value).map((char) => char.codePointAt(0))),
  afterAssignment,
  afterAssignmentCodePoints: Array.from(afterAssignment).map((char) => char.codePointAt(0)),
  selectedIndex: selector.selectedIndex,
}};
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-url", default="http://127.0.0.1:4173/")
    parser.add_argument("--webdriver", default="http://127.0.0.1:4444")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    output = Path(args.output)
    binary = os.environ.get("WINDS_T143_WEBKIT_BINARY", "/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/MiniBrowser")
    browser_name = os.environ.get("WINDS_T143_WEBKIT_BROWSER_NAME", "MiniBrowser")
    browser_arg = os.environ.get("WINDS_T143_WEBKIT_ARGUMENT", "--automation")
    result: dict[str, Any] = {
        "schema": "winds-t143-selection-probe-v1",
        "purpose": "Diagnostic only; never contributes to T143 qualification checks or sample floors.",
        "browser_name": browser_name,
        "browser_binary": binary,
        "browser_argument": browser_arg,
        "attempts": [],
    }
    session: str | None = None
    try:
        session_value = request_json(
            args.webdriver,
            "POST",
            "/session",
            {
                "capabilities": {
                    "alwaysMatch": {
                        "browserName": browser_name,
                        "webkitgtk:browserOptions": {"binary": binary, "args": [browser_arg]},
                    }
                }
            },
        )
        if not isinstance(session_value, dict) or not session_value.get("sessionId"):
            raise RuntimeError(f"WebDriver did not return a session id: {session_value!r}")
        session = session_value["sessionId"]
        result["webdriver_capabilities"] = session_value.get("capabilities", {})
        request_json(args.webdriver, "POST", f"/session/{session}/url", {"url": f"{args.base_url}?tool=chat&theme=dark"})
        wait_until(
            args.webdriver,
            session,
            "return document.readyState === 'complete' && Boolean(document.querySelector('.winds-app'));",
            "Winds shell",
        )
        wait_until(
            args.webdriver,
            session,
            "return (document.querySelector('#chat-session-target')?.options.length ?? 0) >= 3;",
            "Chat Session selector with three options",
        )
        result["ready_snapshot"] = execute(args.webdriver, session, SNAPSHOT_SCRIPT)

        session_a = "fixture-winds\u0000fixture-session-a"
        session_b = "fixture-winds\u0000fixture-session-b"
        for index, target in enumerate((session_b, session_a, session_b, session_a, session_b, session_a)):
            attempt: dict[str, Any] = {"index": index, "target": target}
            attempt["before"] = execute(args.webdriver, session, SNAPSHOT_SCRIPT)
            attempt["dispatch"] = execute(args.webdriver, session, selection_attempt_script(target))
            deadline = time.monotonic() + 2.0
            committed = False
            while time.monotonic() < deadline:
                current = execute(args.webdriver, session, "return document.querySelector('#chat-session-target')?.value ?? null;")
                if current == target:
                    committed = True
                    break
                time.sleep(0.05)
            attempt["committed_within_2s"] = committed
            attempt["after"] = execute(args.webdriver, session, SNAPSHOT_SCRIPT)
            result["attempts"].append(attempt)
            if not committed:
                break
        result["probe_completed"] = True
    except Exception as error:
        result["probe_completed"] = False
        result["error_type"] = type(error).__name__
        result["error"] = str(error)
        if session:
            try:
                result["failure_snapshot"] = execute(args.webdriver, session, SNAPSHOT_SCRIPT)
            except Exception as snapshot_error:
                result["failure_snapshot_error"] = str(snapshot_error)
    finally:
        if session:
            try:
                request_json(args.webdriver, "DELETE", f"/session/{session}")
            except Exception as error:
                result["cleanup_warning"] = str(error)
        output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
