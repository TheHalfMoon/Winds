use super::Result;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::process::Command;

#[allow(
    dead_code,
    reason = "Spec 003 T049 backend API; WSL terminal launch callers land in T052/T057"
)]
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WslDistribution {
    pub name: String,
    pub state: String,
    pub version: u8,
}

#[cfg(windows)]
#[allow(
    dead_code,
    reason = "Spec 003 T049 backend API; WSL terminal launch callers land in T052/T057"
)]
pub fn discover_wsl_distributions() -> Result<Vec<WslDistribution>> {
    let wsl = system_wsl_executable()?;
    let quiet = run_wsl(&wsl, ["--list", "--quiet"])?;
    let names = parse_quiet_names(&quiet)?;
    if names.is_empty() {
        return Ok(Vec::new());
    }

    let verbose = run_wsl(&wsl, ["--list", "--verbose"])?;
    reconcile_verbose_rows(names, &verbose)
}

#[cfg(not(windows))]
#[allow(
    dead_code,
    reason = "Spec 003 T049 backend API; WSL terminal launch callers land in T052/T057"
)]
pub fn discover_wsl_distributions() -> Result<Vec<WslDistribution>> {
    Err("WSL discovery is only available on a native Windows host".into())
}

#[cfg(windows)]
fn system_wsl_executable() -> Result<PathBuf> {
    let system_root = std::env::var_os("SystemRoot")
        .ok_or("WSL discovery unavailable: SystemRoot is not defined")?;
    let executable = PathBuf::from(system_root).join("System32").join("wsl.exe");
    if !executable.is_absolute() {
        return Err("WSL discovery unavailable: SystemRoot is not absolute".into());
    }
    let metadata = fs::metadata(&executable).map_err(|error| {
        format!(
            "WSL discovery unavailable: {} cannot be inspected: {error}",
            executable.display()
        )
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "WSL discovery unavailable: {} is not a file",
            executable.display()
        )
        .into());
    }
    Ok(executable)
}

#[cfg(windows)]
fn run_wsl<const N: usize>(executable: &PathBuf, args: [&str; N]) -> Result<Vec<u8>> {
    let output = Command::new(executable).args(args).output().map_err(|error| {
        format!(
            "WSL discovery unavailable: failed to execute {}: {error}",
            executable.display()
        )
    })?;
    if !output.status.success() {
        let stderr = decode_wsl_text(&output.stderr)
            .unwrap_or_else(|_| String::from_utf8_lossy(&output.stderr).into_owned());
        return Err(format!(
            "WSL discovery command failed with status {}: {}",
            output.status,
            stderr.trim()
        )
        .into());
    }
    if output.stdout.len() > 1024 * 1024 {
        return Err("WSL discovery output exceeded the 1 MiB safety bound".into());
    }
    Ok(output.stdout)
}

fn parse_quiet_names(bytes: &[u8]) -> Result<Vec<String>> {
    let text = decode_wsl_text(bytes)?;
    let mut seen = BTreeSet::new();
    let mut names = Vec::new();
    for line in text.lines() {
        let name = line.trim();
        if name.is_empty() {
            continue;
        }
        if !seen.insert(name.to_owned()) {
            return Err(format!("WSL discovery is ambiguous: duplicate distribution name {name:?}").into());
        }
        names.push(name.to_owned());
    }
    Ok(names)
}

fn reconcile_verbose_rows(
    names: Vec<String>,
    verbose_bytes: &[u8],
) -> Result<Vec<WslDistribution>> {
    let text = decode_wsl_text(verbose_bytes)?;
    let mut longest_names = names.clone();
    longest_names.sort_by_key(|name| std::cmp::Reverse(name.len()));

    let mut rows = BTreeMap::new();
    for line in text.lines() {
        let mut row = line.trim_start();
        if let Some(without_default_marker) = row.strip_prefix('*') {
            row = without_default_marker.trim_start();
        }
        if row.is_empty() {
            continue;
        }

        let matched_name = longest_names.iter().find(|name| {
            row.strip_prefix(name.as_str()).is_some_and(|suffix| {
                suffix
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace)
            })
        });

        let Some(name) = matched_name else {
            if verbose_line_looks_like_distribution(row) {
                return Err(format!(
                    "WSL discovery is ambiguous: verbose output contains a distribution absent from --list --quiet: {row:?}"
                )
                .into());
            }
            continue;
        };

        let suffix = row[name.len()..].trim();
        let mut fields: Vec<&str> = suffix.split_whitespace().collect();
        let version_text = fields
            .pop()
            .ok_or_else(|| format!("WSL verbose row has no version for distribution {name:?}"))?;
        let version = parse_wsl_version(version_text, name)?;
        let state = fields.join(" ");
        if state.is_empty() {
            return Err(format!("WSL verbose row has no state for distribution {name:?}").into());
        }

        let distribution = WslDistribution {
            name: name.clone(),
            state,
            version,
        };
        if rows.insert(name.clone(), distribution).is_some() {
            return Err(format!(
                "WSL discovery is ambiguous: duplicate verbose row for distribution {name:?}"
            )
            .into());
        }
    }

    let mut distributions = Vec::with_capacity(names.len());
    for name in names {
        let distribution = rows.remove(&name).ok_or_else(|| {
            format!(
                "WSL discovery is ambiguous: --list --verbose did not report distribution {name:?}"
            )
        })?;
        distributions.push(distribution);
    }
    distributions.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(distributions)
}

fn parse_wsl_version(value: &str, name: &str) -> Result<u8> {
    match value.parse::<u8>() {
        Ok(version @ (1 | 2)) => Ok(version),
        _ => Err(format!(
            "WSL verbose row has unsupported version {value:?} for distribution {name:?}"
        )
        .into()),
    }
}

fn verbose_line_looks_like_distribution(row: &str) -> bool {
    row.split_whitespace()
        .next_back()
        .is_some_and(|field| matches!(field.parse::<u8>(), Ok(1 | 2)))
}

fn decode_wsl_text(bytes: &[u8]) -> Result<String> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.contains(&0) {
        let bytes = bytes.strip_prefix(&[0xff, 0xfe]).unwrap_or(bytes);
        if bytes.len() % 2 != 0 {
            return Err("WSL discovery returned malformed UTF-16LE output".into());
        }
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        let text = String::from_utf16(&units)
            .map_err(|error| format!("WSL discovery returned invalid UTF-16LE: {error}"))?;
        if text.contains('\0') {
            return Err("WSL discovery returned embedded NUL text".into());
        }
        return Ok(text);
    }

    let text = std::str::from_utf8(bytes)
        .map_err(|error| format!("WSL discovery returned invalid text encoding: {error}"))?;
    Ok(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{decode_wsl_text, parse_quiet_names, reconcile_verbose_rows};

    fn utf16le(value: &str) -> Vec<u8> {
        value
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>()
    }

    #[test]
    fn reconciles_exact_quiet_names_with_verbose_state_and_version() {
        let quiet = utf16le("Ubuntu\r\nMy Distro\r\nUbuntu 24.04\r\n");
        let verbose = utf16le(
            "  NAME                   STATE             VERSION\r\n* Ubuntu                  Running           2\r\n  My Distro               Stopped           1\r\n  Ubuntu 24.04            Running           2\r\n",
        );

        let names = parse_quiet_names(&quiet).unwrap();
        let distributions = reconcile_verbose_rows(names, &verbose).unwrap();

        assert_eq!(distributions.len(), 3);
        assert_eq!(distributions[0].name, "My Distro");
        assert_eq!(distributions[0].state, "Stopped");
        assert_eq!(distributions[0].version, 1);
        assert_eq!(distributions[1].name, "Ubuntu");
        assert_eq!(distributions[1].state, "Running");
        assert_eq!(distributions[1].version, 2);
        assert_eq!(distributions[2].name, "Ubuntu 24.04");
    }

    #[test]
    fn preserves_localized_multiword_state_without_parsing_headers() {
        let quiet = b"Equipe Dev\r\n";
        let verbose = b"  NOM                    ETAT              VERSION\r\n* Equipe Dev              En cours          2\r\n";

        let names = parse_quiet_names(quiet).unwrap();
        let distributions = reconcile_verbose_rows(names, verbose).unwrap();

        assert_eq!(distributions.len(), 1);
        assert_eq!(distributions[0].name, "Equipe Dev");
        assert_eq!(distributions[0].state, "En cours");
        assert_eq!(distributions[0].version, 2);
    }

    #[test]
    fn rejects_verbose_identity_drift_between_supported_wsl_queries() {
        let names = parse_quiet_names(b"Ubuntu\r\n").unwrap();
        let error = reconcile_verbose_rows(
            names,
            b"  NAME        STATE      VERSION\r\n  Debian      Running    2\r\n",
        )
        .unwrap_err();

        assert!(error.to_string().contains("absent from --list --quiet"));
    }

    #[test]
    fn rejects_missing_or_duplicate_verbose_rows() {
        let names = parse_quiet_names(b"Ubuntu\r\n").unwrap();
        let missing = reconcile_verbose_rows(names.clone(), b"NAME STATE VERSION\r\n").unwrap_err();
        assert!(missing.to_string().contains("did not report distribution"));

        let duplicate = reconcile_verbose_rows(
            names,
            b"Ubuntu Running 2\r\nUbuntu Stopped 2\r\n",
        )
        .unwrap_err();
        assert!(duplicate.to_string().contains("duplicate verbose row"));
    }

    #[test]
    fn empty_quiet_list_is_unambiguously_empty() {
        let names = parse_quiet_names(&utf16le("\r\n")).unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn decodes_utf8_and_utf16le_and_rejects_malformed_utf16le() {
        assert_eq!(decode_wsl_text(b"Ubuntu\r\n").unwrap(), "Ubuntu\r\n");
        assert_eq!(decode_wsl_text(&utf16le("Ubuntu\r\n")).unwrap(), "Ubuntu\r\n");

        let error = decode_wsl_text(&[b'U', 0, b'b']).unwrap_err();
        assert!(error.to_string().contains("malformed UTF-16LE"));
    }

    #[cfg(not(windows))]
    #[test]
    fn live_discovery_fails_explicitly_off_native_windows() {
        let error = super::discover_wsl_distributions().unwrap_err();
        assert!(error.to_string().contains("native Windows host"));
    }
}
