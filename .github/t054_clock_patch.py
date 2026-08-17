from pathlib import Path
import sys


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one marker, found {count}")
    return text.replace(old, new, 1)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: t054_clock_patch.py <repo-root>")
    root = Path(sys.argv[1]).resolve()
    command = root / "src/command.rs"
    text = command.read_text(encoding="utf-8")

    text = replace_once(
        text,
        "            let failed_unix_ms = unix_ms().ok();\n",
        "            let failed_unix_ms = trustworthy_wall_time_after(requested_unix_ms, None);\n",
        "spawn-failure wall time",
    )
    text = replace_once(
        text,
        "    let started_unix_ms = unix_ms().ok();\n",
        "    let started_unix_ms = trustworthy_wall_time_after(requested_unix_ms, None);\n",
        "start wall time",
    )
    text = replace_once(
        text,
        "        let ended_unix_ms = unix_ms().ok();\n        let repair = if cleanup_proven {\n",
        "        let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);\n        let repair = if cleanup_proven {\n",
        "RUNNING-persistence repair wall time",
    )
    text = replace_once(
        text,
        "            let ended_unix_ms = unix_ms().ok();\n            let persist = if cleanup_proven {\n",
        "            let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);\n            let persist = if cleanup_proven {\n",
        "wait-failure repair wall time",
    )
    text = replace_once(
        text,
        "    let ended_unix_ms = unix_ms().ok();\n    let exit_code = status.code();\n",
        "    let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);\n    let exit_code = status.code();\n",
        "natural-exit wall time",
    )

    marker = """fn unix_ms() -> Result<i64> {\n    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();\n    Ok(i64::try_from(millis)?)\n}\n"""
    replacement = """fn trustworthy_wall_time_after(\n    requested_unix_ms: i64,\n    started_unix_ms: Option<i64>,\n) -> Option<i64> {\n    non_regressing_wall_time(unix_ms().ok(), requested_unix_ms, started_unix_ms)\n}\n\nfn non_regressing_wall_time(\n    candidate_unix_ms: Option<i64>,\n    requested_unix_ms: i64,\n    started_unix_ms: Option<i64>,\n) -> Option<i64> {\n    candidate_unix_ms.filter(|candidate| {\n        *candidate >= requested_unix_ms\n            && started_unix_ms.is_none_or(|started| *candidate >= started)\n    })\n}\n\nfn unix_ms() -> Result<i64> {\n    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();\n    Ok(i64::try_from(millis)?)\n}\n"""
    text = replace_once(text, marker, replacement, "wall-clock trust helper")
    text = replace_once(
        text,
        "    use super::{ExplicitCommandRequest, run_explicit_command};",
        "    use super::{ExplicitCommandRequest, non_regressing_wall_time, run_explicit_command};",
        "clock regression test import",
    )
    anchor = """    #[test]\n    fn explicit_command_records_structured_intent_and_observed_exit() {\n"""
    regression_test = """    #[test]\n    fn regressed_wall_clock_is_discarded_instead_of_corrupting_lifecycle_truth() {\n        assert_eq!(non_regressing_wall_time(Some(9), 10, None), None);\n        assert_eq!(non_regressing_wall_time(Some(10), 10, Some(11)), None);\n        assert_eq!(non_regressing_wall_time(Some(12), 10, Some(11)), Some(12));\n        assert_eq!(non_regressing_wall_time(None, 10, Some(11)), None);\n    }\n\n""" + anchor
    text = replace_once(text, anchor, regression_test, "clock regression test")
    command.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
