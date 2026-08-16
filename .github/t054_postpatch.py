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

    command = root / "src/command.rs"
    text = command.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "        let mut store = Store::open(&home).unwrap();\n",
        "        let store = Store::open(&home).unwrap();\n",
        "unnecessary mutable Store test helper binding",
    )
    command.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
