from pathlib import Path
import sys


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one marker, found {count}")
    return text.replace(old, new, 1)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: t054_postpatch.py <repo-root>")
    root = Path(sys.argv[1]).resolve()

    migration = root / "migrations/0004_shell_commands.sql"
    text = migration.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "\nCREATE INDEX IF NOT EXISTS idx_shell_commands_executable\n    ON shell_commands(executable, execution_id);\n",
        "\n",
        "speculative shell-command executable index",
    )
    migration.write_text(text, encoding="utf-8")

    store = root / "src/store.rs"
    text = store.read_text(encoding="utf-8")
    marker = "    pub fn mark_shell_command_exited(\n"
    ownership_method = r'''    pub fn mark_shell_command_ownership_lost(
        &mut self,
        execution_id: &str,
        observed_unix_ms: i64,
    ) -> Result<()> {
        let tx = self.connection.transaction()?;
        let (status, requested_unix_ms, _started_unix_ms) =
            shell_command_execution_state(&tx, execution_id)?;
        if !matches!(
            status,
            ExecutionStatus::Requested | ExecutionStatus::Running
        ) {
            return Err(format!(
                "shell-command ownership cannot be lost from persisted state {}: {execution_id}",
                status.as_str()
            )
            .into());
        }
        if observed_unix_ms < requested_unix_ms {
            return Err(
                "shell-command ownership-loss observation cannot precede its request time".into(),
            );
        }
        let updated = tx.execute(
            "UPDATE executions
             SET status = ?2, status_source = ?3,
                 ended_unix_ms = NULL, duration_ms = NULL
             WHERE execution_id = ?1 AND status IN (?4, ?5)",
            params![
                execution_id,
                ExecutionStatus::OwnershipLost.as_str(),
                FactSource::WindsObserved.as_str(),
                ExecutionStatus::Requested.as_str(),
                ExecutionStatus::Running.as_str(),
            ],
        )?;
        if updated != 1 {
            return Err("shell-command ownership-loss transition lost its non-final row".into());
        }
        insert_execution_event(
            &tx,
            execution_id,
            "ShellCommandOwnershipLost",
            FactSource::WindsObserved,
            observed_unix_ms,
        )?;
        tx.commit()?;
        Ok(())
    }

'''
    text = replace_once(text, marker, ownership_method + marker, "shell-command ownership-loss method")
    store.write_text(text, encoding="utf-8")

    command = root / "src/command.rs"
    text = command.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "        let mut store = Store::open(&home).unwrap();\n",
        "        let store = Store::open(&home).unwrap();\n",
        "unnecessary mutable Store test helper binding",
    )
    text = replace_once(
        text,
        '''        let repair = if cleanup_proven {\n            store.mark_shell_command_start_persistence_failed(\n                request.execution_id,\n                started_unix_ms,\n                ended_unix_ms,\n            )\n        } else {\n            Ok(())\n        };\n        let repair_note = match repair {\n            Ok(()) if cleanup_proven => "interrupted cleanup state persisted".to_owned(),\n            Ok(()) => "cleanup was not proven; request remains non-final for restart reconciliation".to_owned(),\n            Err(error) => format!("cleanup-state persistence also failed: {error}"),\n        };\n''',
        '''        let repair = if cleanup_proven {\n            store.mark_shell_command_start_persistence_failed(\n                request.execution_id,\n                started_unix_ms,\n                ended_unix_ms,\n            )\n        } else {\n            store.mark_shell_command_ownership_lost(request.execution_id, ended_unix_ms)\n        };\n        let repair_note = match repair {\n            Ok(()) if cleanup_proven => "interrupted cleanup state persisted".to_owned(),\n            Ok(()) => "cleanup was not proven; ownership-loss state persisted".to_owned(),\n            Err(error) => format!("cleanup-state persistence also failed: {error}"),\n        };\n''',
        "RUNNING-persistence failure ownership-loss repair",
    )
    text = replace_once(
        text,
        '''            let persist = if cleanup_proven {\n                store.mark_shell_command_interrupted(request.execution_id, ended_unix_ms)\n            } else {\n                Ok(())\n            };\n''',
        '''            let persist = if cleanup_proven {\n                store.mark_shell_command_interrupted(request.execution_id, ended_unix_ms)\n            } else {\n                store.mark_shell_command_ownership_lost(request.execution_id, ended_unix_ms)\n            };\n''',
        "wait failure ownership-loss repair",
    )
    command.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
