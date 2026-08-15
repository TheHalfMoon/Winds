#!/usr/bin/env python3
"""Collect the locked Cargo dependency license files for a release bundle."""

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

LICENSE_PREFIXES = (
    "LICENSE",
    "COPYING",
    "NOTICE",
    "UNLICENSE",
    "COPYRIGHT",
)

REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
LICENSE_OVERRIDES = {
    ("rsqlite-vfs", "0.1.1"): REPOSITORY_ROOT
    / "third-party"
    / "licenses"
    / "rsqlite-vfs-0.1.1"
    / "LICENSE",
}


def is_license_file(path: Path) -> bool:
    return path.is_file() and not path.is_symlink() and path.name.upper().startswith(LICENSE_PREFIXES)


def copy_override(name: str, version: str, package_dir: Path) -> bool:
    override = LICENSE_OVERRIDES.get((name, version))
    if override is None:
        return False
    if not override.is_file() or override.is_symlink():
        raise SystemExit(f"invalid license override for {name} {version}: {override}")
    package_dir.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(override, package_dir / "LICENSE")
    print(f"LICENSE_OVERRIDE_USED={name} {version}")
    return True


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: collect_licenses.py METADATA_JSON OUTPUT_DIR", file=sys.stderr)
        return 2

    metadata_path = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    workspace = set(metadata["workspace_members"])
    packages = sorted(
        (package for package in metadata["packages"] if package["id"] not in workspace),
        key=lambda package: (package["name"], package["version"]),
    )

    if output_dir.exists():
        raise SystemExit(f"refusing to overwrite existing license directory: {output_dir}")
    output_dir.mkdir(parents=True)

    inventory_lines = ["name\tversion\tlicense\tsource"]
    missing: list[str] = []
    unicode_notice_seen = False

    for package in packages:
        name = package["name"]
        version = package["version"]
        license_expression = package.get("license") or "<NONE>"
        source = package.get("source") or "<NONE>"
        manifest_path = Path(package["manifest_path"])
        package_root = manifest_path.parent
        package_dir = output_dir / f"{name}-{version}"

        candidates = sorted(
            path
            for path in package_root.rglob("*")
            if is_license_file(path)
        )
        if not candidates:
            if not copy_override(name, version, package_dir):
                missing.append(f"{name} {version}")
                continue
        else:
            for source_path in candidates:
                relative = source_path.relative_to(package_root)
                destination = package_dir / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source_path, destination)
                if name == "unicode-ident" and "unicode" in source_path.name.lower():
                    unicode_notice_seen = True

        inventory_lines.append(f"{name}\t{version}\t{license_expression}\t{source}")

    (output_dir / "INVENTORY.tsv").write_text(
        "\n".join(inventory_lines) + "\n",
        encoding="utf-8",
    )

    if missing:
        raise SystemExit(
            "locked packages missing distributable license/notice files: " + ", ".join(missing)
        )
    if not unicode_notice_seen:
        raise SystemExit("unicode-ident Unicode license/notice file was not collected")

    print(f"THIRD_PARTY_PACKAGE_COUNT={len(packages)}")
    print("THIRD_PARTY_LICENSE_FILES_COMPLETE=YES")
    print("UNICODE_NOTICE_PRESENT=YES")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
