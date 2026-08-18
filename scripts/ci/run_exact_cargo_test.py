#!/usr/bin/env python3

import os
import re
import subprocess
import sys


def fail(message: str) -> None:
    print(f"T063 exact-test guard failed: {message}", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        fail("usage: run_exact_cargo_test.py <expected-test-name> -- <cargo command...>")

    expected = sys.argv[1]
    command = sys.argv[3:]
    if not command or command[0] != "cargo":
        fail("guard only accepts an explicit cargo command")

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

    expected_result = f"test {expected} ... ok"
    if expected_result not in output:
        fail(f"expected successful test line not found: {expected_result!r}")

    summaries = re.findall(
        r"^test result: ok\. 1 passed; 0 failed; \d+ ignored; \d+ measured; \d+ filtered out; finished in .+$",
        output,
        flags=re.MULTILINE,
    )
    if len(summaries) != 1:
        fail(f"expected exactly one one-test success summary, found {len(summaries)}")

    print(f"T063_EXACT_TEST_PROVEN={expected}")


if __name__ == "__main__":
    main()
