#!/usr/bin/env python3

import os
import re
import subprocess
import sys


def fail(message: str) -> None:
    print(f"exact-test guard failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    try:
        separator = sys.argv.index("--", 1)
    except ValueError:
        fail(
            "usage: run_exact_cargo_test.py <expected-test-name> "
            "[--marker-prefix <prefix>] -- <cargo command...>"
        )

    options = sys.argv[1:separator]
    if len(options) == 1:
        expected = options[0]
        marker_prefix = "T063"
    elif len(options) == 3 and options[1] == "--marker-prefix":
        expected = options[0]
        marker_prefix = options[2]
    else:
        fail(
            "usage: run_exact_cargo_test.py <expected-test-name> "
            "[--marker-prefix <prefix>] -- <cargo command...>"
        )

    if not expected:
        fail("expected test name must not be empty")
    if not re.fullmatch(r"[A-Z][A-Z0-9_]*", marker_prefix):
        fail("marker prefix must match [A-Z][A-Z0-9_]*")

    command = sys.argv[separator + 1 :]
    if not command or command[0] != "cargo":
        fail("guard only accepts an explicit cargo command")
    if "--exact" not in command:
        fail("guard requires cargo test harness --exact filtering")

    env = os.environ.copy()
    env["CARGO_TERM_COLOR"] = "never"
    env["NO_COLOR"] = "1"

    completed = subprocess.run(
        command,
        cwd=os.getcwd(),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    output = completed.stdout
    sys.stdout.write(output)

    if completed.returncode != 0:
        raise SystemExit(completed.returncode)

    started = re.findall(
        rf"^test {re.escape(expected)} \.\.\.",
        output,
        flags=re.MULTILINE,
    )
    if len(started) != 1:
        fail(f"expected exactly one test start for {expected!r}, found {len(started)}")

    summaries = re.findall(
        r"^test result: ok\. 1 passed; 0 failed; \d+ ignored; \d+ measured; \d+ filtered out; finished in .+$",
        output,
        flags=re.MULTILINE,
    )
    if len(summaries) != 1:
        fail(f"expected exactly one one-test success summary, found {len(summaries)}")

    print(f"{marker_prefix}_EXACT_TEST_PROVEN={expected}")


if __name__ == "__main__":
    main()
