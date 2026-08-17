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
        "use std::time::{SystemTime, UNIX_EPOCH};",
        "use std::time::{Instant, SystemTime, UNIX_EPOCH};",
        "command Instant import",
    )
    text = replace_once(
        text,
        "\n    let mut child = match Command::new(&executable)\n",
        "\n    let monotonic_start = Instant::now();\n    let mut child = match Command::new(&executable)\n",
        "monotonic command start",
    )
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
    old = """        let cleanup_proven = cleanup_owned_child(&mut child);\n        let ended_unix_ms = unix_ms().ok();\n        let repair = if cleanup_proven {\n"""
    new = """        let cleanup_proven = cleanup_owned_child(&mut child);\n        let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);\n        let repair = if cleanup_proven {\n"""
    text = replace_once(text, old, new, "RUNNING-persistence repair wall time")
    old = """            let cleanup_proven = cleanup_owned_child(&mut child);\n            let ended_unix_ms = unix_ms().ok();\n            let persist = if cleanup_proven {\n"""
    new = """            let cleanup_proven = cleanup_owned_child(&mut child);\n            let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);\n            let persist = if cleanup_proven {\n"""
    text = replace_once(text, old, new, "wait-failure repair wall time")
    old = """    let ended_unix_ms = unix_ms().ok();\n    let exit_code = status.code();\n    store\n        .record_shell_command_exit_observation(request.execution_id, exit_code, ended_unix_ms)\n"""
    new = """    let ended_unix_ms = trustworthy_wall_time_after(requested_unix_ms, started_unix_ms);\n    let observed_duration_ms = monotonic_elapsed_ms(monotonic_start)?;\n    let exit_code = status.code();\n    store\n        .record_shell_command_exit_observation(\n            request.execution_id,\n            exit_code,\n            ended_unix_ms,\n            observed_duration_ms,\n        )\n"""
    text = replace_once(text, old, new, "durable exit observation timing")
    old = """fn unix_ms() -> Result<i64> {\n    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();\n    Ok(i64::try_from(millis)?)\n}\n"""
    new = """fn trustworthy_wall_time_after(\n    requested_unix_ms: i64,\n    started_unix_ms: Option<i64>,\n) -> Option<i64> {\n    non_regressing_wall_time(unix_ms().ok(), requested_unix_ms, started_unix_ms)\n}\n\nfn non_regressing_wall_time(\n    candidate_unix_ms: Option<i64>,\n    requested_unix_ms: i64,\n    started_unix_ms: Option<i64>,\n) -> Option<i64> {\n    candidate_unix_ms.filter(|candidate| {\n        *candidate >= requested_unix_ms\n            && started_unix_ms.is_none_or(|started| *candidate >= started)\n    })\n}\n\nfn monotonic_elapsed_ms(start: Instant) -> Result<u64> {\n    Ok(u64::try_from(start.elapsed().as_millis())?)\n}\n\nfn unix_ms() -> Result<i64> {\n    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();\n    Ok(i64::try_from(millis)?)\n}\n"""
    text = replace_once(text, old, new, "timing helpers")
    text = replace_once(
        text,
        "    use super::{ExplicitCommandRequest, run_explicit_command};",
        "    use super::{ExplicitCommandRequest, non_regressing_wall_time, run_explicit_command};",
        "timing test import",
    )
    anchor = """    #[test]\n    fn explicit_command_records_structured_intent_and_observed_exit() {\n"""
    test = """    #[test]\n    fn regressed_wall_clock_is_discarded_instead_of_corrupting_lifecycle_truth() {\n        assert_eq!(non_regressing_wall_time(Some(9), 10, None), None);\n        assert_eq!(non_regressing_wall_time(Some(10), 10, Some(11)), None);\n        assert_eq!(non_regressing_wall_time(Some(12), 10, Some(11)), Some(12));\n        assert_eq!(non_regressing_wall_time(None, 10, Some(11)), None);\n    }\n\n""" + anchor
    text = replace_once(text, anchor, test, "clock-regression test")
    text = replace_once(
        text,
        "        assert!(command.observed_end_unix_ms.is_some());\n",
        "        assert!(command.observed_end_unix_ms.is_some());\n        assert_eq!(command.observed_duration_ms, execution.duration_ms);\n",
        "success duration assertion",
    )
    text = replace_once(
        text,
        "            .record_shell_command_exit_observation(\"command-durable\", Some(9), Some(20))\n",
        "            .record_shell_command_exit_observation(\"command-durable\", Some(9), Some(20), 9)\n",
        "durable exit fixture duration",
    )
    text = replace_once(
        text,
        "        assert_eq!(command.observed_end_unix_ms, Some(20));\n",
        "        assert_eq!(command.observed_end_unix_ms, Some(20));\n        assert_eq!(command.observed_duration_ms, Some(9));\n",
        "durable duration assertion",
    )
    text = replace_once(
        text,
        "    fn observed_exit_without_wall_clock_finalizes_with_unknown_timing() {\n",
        "    fn observed_exit_without_wall_clock_keeps_monotonic_duration() {\n",
        "unknown wall-clock test name",
    )
    text = replace_once(
        text,
        "            .record_shell_command_exit_observation(\"command-clock-unknown\", Some(0), None)\n",
        "            .record_shell_command_exit_observation(\"command-clock-unknown\", Some(0), None, 9)\n",
        "unknown wall-clock fixture duration",
    )
    text = replace_once(
        text,
        "        assert_eq!(execution.duration_ms, None);\n        let command = store.load_shell_command(\"command-clock-unknown\").unwrap();\n",
        "        assert_eq!(execution.duration_ms, Some(9));\n        let command = store.load_shell_command(\"command-clock-unknown\").unwrap();\n",
        "unknown wall-clock execution duration",
    )
    text = replace_once(
        text,
        "        assert_eq!(command.observed_end_unix_ms, None);\n    }\n}\n",
        "        assert_eq!(command.observed_end_unix_ms, None);\n        assert_eq!(command.observed_duration_ms, Some(9));\n    }\n}\n",
        "unknown wall-clock durable duration assertion",
    )
    command.write_text(text, encoding="utf-8")

    migration = root / "migrations/0004_shell_commands.sql"
    text = migration.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "    observed_end_unix_ms INTEGER,\n    CHECK (\n        (exit_source IS NULL AND exit_code IS NULL AND observed_end_unix_ms IS NULL)\n        OR exit_source IS NOT NULL\n    )\n",
        "    observed_end_unix_ms INTEGER,\n    observed_duration_ms INTEGER,\n    CHECK (\n        (exit_source IS NULL AND exit_code IS NULL AND observed_end_unix_ms IS NULL AND observed_duration_ms IS NULL)\n        OR (exit_source IS NOT NULL AND observed_duration_ms IS NOT NULL AND observed_duration_ms >= 0)\n    )\n",
        "shell-command durable duration schema",
    )
    migration.write_text(text, encoding="utf-8")

    domain = root / "src/domain.rs"
    text = domain.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "    pub observed_end_unix_ms: Option<i64>,\n}",
        "    pub observed_end_unix_ms: Option<i64>,\n    pub observed_duration_ms: Option<u64>,\n}",
        "shell-command domain duration",
    )
    domain.write_text(text, encoding="utf-8")

    store = root / "src/store.rs"
    text = store.read_text(encoding="utf-8")
    old = """    pub fn record_shell_command_exit_observation(\n        &mut self,\n        execution_id: &str,\n        exit_code: Option<i32>,\n        observed_end_unix_ms: Option<i64>,\n    ) -> Result<()> {\n"""
    new = """    pub fn record_shell_command_exit_observation(\n        &mut self,\n        execution_id: &str,\n        exit_code: Option<i32>,\n        observed_end_unix_ms: Option<i64>,\n        observed_duration_ms: u64,\n    ) -> Result<()> {\n"""
    text = replace_once(text, old, new, "exit observation signature")
    text = replace_once(
        text,
        "        validate_optional_command_times(requested_unix_ms, started_unix_ms, observed_end_unix_ms)?;\n        let updated = tx.execute(\n",
        "        validate_optional_command_times(requested_unix_ms, started_unix_ms, observed_end_unix_ms)?;\n        let observed_duration_ms = i64::try_from(observed_duration_ms)?;\n        let updated = tx.execute(\n",
        "exit observation duration conversion",
    )
    text = replace_once(
        text,
        "            \"UPDATE shell_commands\n             SET exit_code = ?2, exit_source = ?3, observed_end_unix_ms = ?4\n             WHERE execution_id = ?1 AND exit_source IS NULL\",\n",
        "            \"UPDATE shell_commands\n             SET exit_code = ?2, exit_source = ?3, observed_end_unix_ms = ?4,\n                 observed_duration_ms = ?5\n             WHERE execution_id = ?1 AND exit_source IS NULL\",\n",
        "exit observation SQL",
    )
    text = replace_once(
        text,
        "                observed_end_unix_ms,\n            ],\n",
        "                observed_end_unix_ms,\n                observed_duration_ms,\n            ],\n",
        "exit observation SQL params",
    )
    text = replace_once(
        text,
        "                        c.exit_code, c.exit_source, c.observed_end_unix_ms\n",
        "                        c.exit_code, c.exit_source, c.observed_end_unix_ms,\n                        c.observed_duration_ms\n",
        "finalization durable duration select",
    )
    text = replace_once(
        text,
        "                        row.get::<_, Option<i64>>(5)?,\n                    ))\n",
        "                        row.get::<_, Option<i64>>(5)?,\n                        row.get::<_, Option<i64>>(6)?,\n                    ))\n",
        "finalization durable duration row",
    )
    text = replace_once(
        text,
        "        validate_optional_command_times(row.1, row.2, row.5)?;\n        let duration_ms = optional_duration_ms(row.2, row.5)?;\n",
        "        validate_optional_command_times(row.1, row.2, row.5)?;\n        let duration_ms = row\n            .6\n            .ok_or(\"shell-command completion requires a durable observed duration\")?;\n        if duration_ms < 0 {\n            return Err(\"shell-command observed duration cannot be negative\".into());\n        }\n",
        "finalization durable duration use",
    )
    text = replace_once(
        text,
        "                \"SELECT execution_id, executable, arguments_json, command_source,\n                        requested_cwd, cwd_source, exit_code, exit_source, observed_end_unix_ms\n",
        "                \"SELECT execution_id, executable, arguments_json, command_source,\n                        requested_cwd, cwd_source, exit_code, exit_source, observed_end_unix_ms,\n                        observed_duration_ms\n",
        "shell-command load duration select",
    )
    text = replace_once(
        text,
        "                        row.get::<_, Option<i64>>(8)?,\n                    ))\n",
        "                        row.get::<_, Option<i64>>(8)?,\n                        row.get::<_, Option<i64>>(9)?,\n                    ))\n",
        "shell-command load duration row",
    )
    text = replace_once(
        text,
        "            observed_end_unix_ms: row.8,\n        })\n",
        "            observed_end_unix_ms: row.8,\n            observed_duration_ms: row.9.map(u64::try_from).transpose()?,\n        })\n",
        "shell-command record duration",
    )
    store.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
