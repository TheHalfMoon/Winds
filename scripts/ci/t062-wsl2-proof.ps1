Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Invoke-NativeResult {
    param(
        [Parameter(Mandatory = $true)][string]$File,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [ValidateRange(1, 600000)][int]$TimeoutMilliseconds = 120000
    )

    $startInfo = [System.Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $File
    $startInfo.UseShellExecute = $false
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in $Arguments) {
        [void]$startInfo.ArgumentList.Add($argument)
    }

    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $startInfo
    if (-not $process.Start()) {
        throw "failed to start native command: $File"
    }
    $stdoutTask = $process.StandardOutput.ReadToEndAsync()
    $stderrTask = $process.StandardError.ReadToEndAsync()
    if (-not $process.WaitForExit($TimeoutMilliseconds)) {
        try {
            if (-not $process.HasExited) {
                $process.Kill($true)
            }
        }
        catch {
            Write-Warning "failed to kill timed-out native command $File: $_"
        }
        if (-not $process.WaitForExit(2000)) {
            $process.Dispose()
            throw "native command timed out after ${TimeoutMilliseconds}ms and could not be reaped: $File $($Arguments -join ' ')"
        }
        $stdout = $stdoutTask.GetAwaiter().GetResult().Trim()
        $stderr = $stderrTask.GetAwaiter().GetResult().Trim()
        $process.Dispose()
        throw "native command timed out after ${TimeoutMilliseconds}ms: $File $($Arguments -join ' ')`nstdout:`n$stdout`nstderr:`n$stderr"
    }
    $stdout = $stdoutTask.GetAwaiter().GetResult().Trim()
    $stderr = $stderrTask.GetAwaiter().GetResult().Trim()
    $exitCode = $process.ExitCode
    $process.Dispose()

    return [pscustomobject]@{
        ExitCode = $exitCode
        Stdout = $stdout
        Stderr = $stderr
    }
}

function Invoke-Captured {
    param(
        [Parameter(Mandatory = $true)][string]$File,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [ValidateRange(1, 600000)][int]$TimeoutMilliseconds = 120000
    )

    $result = Invoke-NativeResult -File $File -Arguments $Arguments -TimeoutMilliseconds $TimeoutMilliseconds
    if ($result.ExitCode -ne 0) {
        throw "command failed ($($result.ExitCode)): $File $($Arguments -join ' ')`nstdout:`n$($result.Stdout)`nstderr:`n$($result.Stderr)"
    }
    if (-not [string]::IsNullOrWhiteSpace($result.Stderr)) {
        Write-Host "native command diagnostic ($File): $($result.Stderr)"
    }
    return $result.Stdout
}

function Invoke-ProductionWslBackendProof {
    param([Parameter(Mandatory = $true)][ValidateSet("MAPPED", "FALLBACK")][string]$ExpectedCwd)

    $env:WINDS_T062_EXPECT_CWD = $ExpectedCwd
    try {
        Invoke-Captured "cargo.exe" @(
            "test",
            "--locked",
            "--bin", "winds",
            "t062_real_wsl_backend_launch_is_opt_in_and_uses_production_path",
            "--",
            "--test-threads=1"
        ) | Out-Null
    }
    finally {
        $env:WINDS_T062_EXPECT_CWD = $null
    }
}

function Resolve-CanonicalWindowsPath {
    param([Parameter(Mandatory = $true)][string]$Path)

    $resolved = (Resolve-Path -LiteralPath $Path).Path
    return [System.IO.Path]::GetFullPath($resolved).TrimEnd('\')
}

function Assert-Equal {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [AllowEmptyString()][string]$Actual,
        [AllowEmptyString()][string]$Expected
    )

    if ($Actual -cne $Expected) {
        throw "$Label mismatch: actual=$Actual expected=$Expected"
    }
}

function Assert-WindowsPathEqual {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][string]$Actual,
        [Parameter(Mandatory = $true)][string]$Expected
    )

    $actualCanonical = Resolve-CanonicalWindowsPath $Actual
    $expectedCanonical = Resolve-CanonicalWindowsPath $Expected
    if (-not [string]::Equals($actualCanonical, $expectedCanonical, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "$Label mismatch: actual=$actualCanonical expected=$expectedCanonical"
    }
}

function Limit-Diagnostic {
    param([AllowEmptyString()][string]$Value)

    if ($Value.Length -le 4096) {
        return $Value
    }
    return $Value.Substring(0, 4096) + "...[truncated]"
}

function Wait-ForMappedWorkspaceMismatch {
    param(
        [Parameter(Mandatory = $true)][string]$Distribution,
        [Parameter(Mandatory = $true)][string]$LinuxWorkspaceRoot
    )

    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    $marker = "/tmp/winds-t062-observed-cwd"
    $lastDiagnostic = ""
    do {
        $remainingMilliseconds = [int][Math]::Floor(($deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if ($remainingMilliseconds -le 0) {
            break
        }
        $arguments = @(
            "--distribution", $Distribution,
            "--user", "root",
            "--cd", $LinuxWorkspaceRoot,
            "--exec", "/bin/sh", "-c", "pwd -P > $marker"
        )
        $result = Invoke-NativeResult -File "wsl.exe" -Arguments $arguments -TimeoutMilliseconds ([Math]::Min(5000, $remainingMilliseconds))
        $diagnostic = Limit-Diagnostic ((@(
            $result.Stderr,
            $result.Stdout
        ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }) -join "`n")

        if ($result.ExitCode -ne 0) {
            return [pscustomobject]@{
                Behavior = "CD_REJECTED"
                ExitCode = $result.ExitCode
                Diagnostic = $diagnostic
                ObservedCwd = $null
            }
        }

        $remainingMilliseconds = [int][Math]::Floor(($deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if ($remainingMilliseconds -le 0) {
            $lastDiagnostic = "mapped probe completed at deadline; diagnostic=$diagnostic"
            break
        }
        $markerResult = Invoke-NativeResult -File "wsl.exe" -Arguments @(
            "--distribution", $Distribution,
            "--user", "root",
            "--exec", "/bin/cat", $marker
        ) -TimeoutMilliseconds ([Math]::Min(5000, $remainingMilliseconds))
        if ($markerResult.ExitCode -eq 0) {
            $observedCwd = $markerResult.Stdout.Trim()
            if ($observedCwd -cne $LinuxWorkspaceRoot) {
                return [pscustomobject]@{
                    Behavior = "VISIBLE_CWD_MISMATCH"
                    ExitCode = $result.ExitCode
                    Diagnostic = $diagnostic
                    ObservedCwd = $observedCwd
                }
            }
            $lastDiagnostic = "mapped workspace still active: cwd=$observedCwd; diagnostic=$diagnostic"
        }
        else {
            $markerDiagnostic = Limit-Diagnostic ((@(
                $markerResult.Stderr,
                $markerResult.Stdout
            ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }) -join "`n")
            $lastDiagnostic = "mapped probe succeeded but cwd marker could not be read: $markerDiagnostic"
        }

        Start-Sleep -Milliseconds 250
    } while ([DateTime]::UtcNow -lt $deadline)

    throw "WSL did not expose mapped-workspace mismatch within the bounded probe window: $lastDiagnostic"
}

$distro = $env:WINDS_T062_WSL_DISTRO
if ([string]::IsNullOrWhiteSpace($distro)) {
    throw "WINDS_T062_WSL_DISTRO must name the exact installed WSL distribution"
}
$tempRoot = $env:RUNNER_TEMP
if ([string]::IsNullOrWhiteSpace($tempRoot)) {
    throw "RUNNER_TEMP must point at the runner-owned scratch directory"
}

$repo = Resolve-CanonicalWindowsPath (Invoke-Captured "git.exe" @("rev-parse", "--show-toplevel"))
$hostHead = Invoke-Captured "git.exe" @("-C", $repo, "rev-parse", "--verify", "HEAD^{commit}")
$hostCommon = Resolve-CanonicalWindowsPath (Invoke-Captured "git.exe" @("-C", $repo, "rev-parse", "--path-format=absolute", "--git-common-dir"))
$windsHome = Join-Path $tempRoot ("winds-t062-home-" + $hostHead.Substring(0, 12))
if (Test-Path -LiteralPath $windsHome) {
    throw "refusing to reuse pre-existing exact-head T062 Winds home: $windsHome"
}

Invoke-Captured "cargo.exe" @("build", "--locked", "--bin", "winds") | Out-Null
$winds = Join-Path $repo "target\debug\winds.exe"
if (-not (Test-Path -LiteralPath $winds -PathType Leaf)) {
    throw "Winds proof binary is missing: $winds"
}

$profilesText = & $winds profiles --repo $repo --home $windsHome
if ($LASTEXITCODE -ne 0) {
    throw "winds profiles failed with exit code $LASTEXITCODE"
}
$profiles = ($profilesText | Out-String) | ConvertFrom-Json
if ($profiles.wsl.availability -ne "AVAILABLE") {
    throw "Winds did not report WSL discovery as AVAILABLE: $($profiles.wsl | ConvertTo-Json -Depth 8 -Compress)"
}
$selected = @($profiles.wsl.distributions | Where-Object { $_.name -ceq $distro })
if ($selected.Count -ne 1) {
    throw "Winds did not discover exactly one selected distribution named $distro"
}
if ([int]$selected[0].version -ne 2) {
    throw "selected distribution is not WSL2: $($selected[0] | ConvertTo-Json -Compress)"
}

Invoke-ProductionWslBackendProof "MAPPED"
$mappedBackendLaunch = "PASS"

$linuxRepo = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--exec", "/usr/bin/wslpath", $repo)
if (-not $linuxRepo.StartsWith("/", [System.StringComparison]::Ordinal)) {
    throw "wslpath did not return an absolute Linux repository path: $linuxRepo"
}

$effectiveCwd = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", $linuxRepo, "--exec", "/bin/pwd", "-P")
$linuxRoot = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", $linuxRepo, "--exec", "/usr/bin/git", "rev-parse", "--show-toplevel")
$linuxCommon = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", $linuxRepo, "--exec", "/usr/bin/git", "rev-parse", "--path-format=absolute", "--git-common-dir")
$linuxHead = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", $linuxRepo, "--exec", "/usr/bin/git", "rev-parse", "--verify", "HEAD^{commit}")
Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", $linuxRepo, "--exec", "/bin/sh", "-c", "exit 0") | Out-Null

$effectiveWindows = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--exec", "/usr/bin/wslpath", "-w", $effectiveCwd)
$rootWindows = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--exec", "/usr/bin/wslpath", "-w", $linuxRoot)
$commonWindows = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--exec", "/usr/bin/wslpath", "-w", $linuxCommon)
Assert-WindowsPathEqual "effective WSL cwd" $effectiveWindows $repo
Assert-WindowsPathEqual "WSL Git worktree root" $rootWindows $repo
Assert-WindowsPathEqual "WSL Git common directory" $commonWindows $hostCommon
Assert-Equal "WSL Git HEAD" $linuxHead $hostHead

$mismatchExitCode = $null
$mismatchBehavior = $null
$mismatchDiagnostic = $null
$observedMismatchCwd = $null
$mappedWorkspaceEquivalenceBroken = $false
$fallbackHome = $null
$fallbackWindows = $null
$fallbackBackendLaunch = $null
$wslConfBackup = "/tmp/winds-t062-wsl-conf-$($hostHead.Substring(0, 12)).bak"
$wslConfOriginalState = Invoke-Captured "wsl.exe" @(
    "--distribution", $distro,
    "--user", "root",
    "--exec", "/bin/sh", "-c",
    "if [ -f /etc/wsl.conf ]; then cp /etc/wsl.conf '$wslConfBackup'; printf PRESENT; else rm -f '$wslConfBackup'; printf ABSENT; fi"
)
if ($wslConfOriginalState -notin @("PRESENT", "ABSENT")) {
    throw "unexpected /etc/wsl.conf snapshot state: $wslConfOriginalState"
}
try {
    Invoke-Captured "wsl.exe" @(
        "--distribution", $distro,
        "--user", "root",
        "--exec", "/bin/sh", "-c",
        "printf '[automount]\nenabled=false\n[interop]\nappendWindowsPath=false\n[user]\ndefault=root\n' > /etc/wsl.conf"
    ) | Out-Null
    Invoke-Captured "wsl.exe" @("--terminate", $distro) | Out-Null

    $mismatchObservation = Wait-ForMappedWorkspaceMismatch $distro $linuxRepo
    $mismatchExitCode = $mismatchObservation.ExitCode
    $mismatchBehavior = $mismatchObservation.Behavior
    $mismatchDiagnostic = $mismatchObservation.Diagnostic
    $observedMismatchCwd = $mismatchObservation.ObservedCwd
    $mappedWorkspaceEquivalenceBroken = $mismatchBehavior -in @("CD_REJECTED", "VISIBLE_CWD_MISMATCH")
    if (-not $mappedWorkspaceEquivalenceBroken) {
        throw "T062 mismatch proof did not establish broken mapped-workspace equivalence"
    }

    Invoke-ProductionWslBackendProof "FALLBACK"
    $fallbackBackendLaunch = "PASS"

    $fallbackHome = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", "~", "--exec", "/bin/pwd", "-P")
    if (-not $fallbackHome.StartsWith("/", [System.StringComparison]::Ordinal)) {
        throw "fallback WSL home is not an absolute Linux path: $fallbackHome"
    }
    if ($fallbackHome -ceq $linuxRepo) {
        throw "fallback WSL home unexpectedly equals the mapped Linux workspace: $fallbackHome"
    }
    $fallbackWindows = Invoke-Captured "wsl.exe" @(
        "--distribution", $distro,
        "--user", "root",
        "--exec", "/usr/bin/wslpath", "-w", $fallbackHome
    )
    $fallbackWindowsComparable = $fallbackWindows.TrimEnd('\')
    $repoComparable = $repo.TrimEnd('\')
    if ([string]::Equals($fallbackWindowsComparable, $repoComparable, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "fallback WSL home unexpectedly maps back to the canonical Windows workspace: $fallbackWindows"
    }
    Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", $fallbackHome, "--exec", "/bin/sh", "-c", "exit 0") | Out-Null
}
finally {
    try {
        $restoreCommand = if ($wslConfOriginalState -ceq "PRESENT") {
            "mv '$wslConfBackup' /etc/wsl.conf"
        }
        else {
            "rm -f /etc/wsl.conf '$wslConfBackup'"
        }
        Invoke-Captured "wsl.exe" @(
            "--distribution", $distro,
            "--user", "root",
            "--exec", "/bin/sh", "-c", $restoreCommand
        ) | Out-Null
        Invoke-Captured "wsl.exe" @("--terminate", $distro) | Out-Null
    }
    catch {
        Write-Warning "T062 cleanup could not restore the original WSL configuration: $_"
    }
}

$summary = [ordered]@{
    schema_version = 1
    evidence = "T062_REAL_WINDOWS_WSL2_INTEGRATION"
    repository_head = $hostHead
    distribution = [ordered]@{
        name = $selected[0].name
        state_at_discovery = $selected[0].state
        version = [int]$selected[0].version
    }
    production_backend = [ordered]@{
        mapped_prepare_and_launch = $mappedBackendLaunch
        fallback_prepare_and_launch = $fallbackBackendLaunch
    }
    mapped_workspace = [ordered]@{
        windows_root = $repo
        linux_root = $linuxRoot
        effective_cwd = $effectiveCwd
        linux_git_common_dir = $linuxCommon
        git_head_oid = $linuxHead
        selected_distribution_launch = "PASS"
    }
    mismatch = [ordered]@{
        automount_disabled = $true
        behavior = $mismatchBehavior
        mapped_cwd_exit_code = $mismatchExitCode
        diagnostic = $mismatchDiagnostic
        observed_cwd = $observedMismatchCwd
        mapped_workspace_equivalence_broken = $mappedWorkspaceEquivalenceBroken
        fallback_home = $fallbackHome
        fallback_windows = $fallbackWindows
        fallback_equivalent_to_mapped_workspace = $false
        fallback_launch = "PASS"
    }
}

$summary | ConvertTo-Json -Depth 8
