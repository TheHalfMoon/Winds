Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Invoke-Captured {
    param(
        [Parameter(Mandatory = $true)][string]$File,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    $output = & $File @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    $text = (@($output) -join "`n").Trim()
    if ($exitCode -ne 0) {
        throw "command failed ($exitCode): $File $($Arguments -join ' ')`n$text"
    }
    return $text
}

function Invoke-ExpectedFailure {
    param(
        [Parameter(Mandatory = $true)][string]$File,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    $output = & $File @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    $text = (@($output) -join "`n").Trim()
    if ($exitCode -eq 0) {
        throw "command unexpectedly succeeded: $File $($Arguments -join ' ')`n$text"
    }
    return [ordered]@{
        exit_code = $exitCode
        diagnostic = $text
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

$distro = $env:WINDS_T062_WSL_DISTRO
if ([string]::IsNullOrWhiteSpace($distro)) {
    throw "WINDS_T062_WSL_DISTRO must name the exact installed WSL distribution"
}

$repo = Resolve-CanonicalWindowsPath (Invoke-Captured "git.exe" @("rev-parse", "--show-toplevel"))
$hostHead = Invoke-Captured "git.exe" @("-C", $repo, "rev-parse", "--verify", "HEAD^{commit}")
$hostCommon = Resolve-CanonicalWindowsPath (Invoke-Captured "git.exe" @("-C", $repo, "rev-parse", "--path-format=absolute", "--git-common-dir"))
$windsHome = Join-Path $env:RUNNER_TEMP "winds-t062-home"
if (Test-Path -LiteralPath $windsHome) {
    Remove-Item -LiteralPath $windsHome -Recurse -Force
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

$mismatch = $null
$fallbackHome = $null
try {
    Invoke-Captured "wsl.exe" @(
        "--distribution", $distro,
        "--user", "root",
        "--exec", "/bin/sh", "-c",
        "printf '[automount]\nenabled=false\n[user]\ndefault=root\n' > /etc/wsl.conf"
    ) | Out-Null
    Invoke-Captured "wsl.exe" @("--terminate", $distro) | Out-Null
    Start-Sleep -Seconds 2
    Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--exec", "/bin/true") | Out-Null

    $mismatch = Invoke-ExpectedFailure "wsl.exe" @(
        "--distribution", $distro,
        "--user", "root",
        "--cd", $linuxRepo,
        "--exec", "/bin/pwd", "-P"
    )
    $fallbackHome = Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", "~", "--exec", "/bin/pwd", "-P")
    if (-not $fallbackHome.StartsWith("/", [System.StringComparison]::Ordinal)) {
        throw "fallback WSL home is not an absolute Linux path: $fallbackHome"
    }
    Invoke-Captured "wsl.exe" @("--distribution", $distro, "--user", "root", "--cd", $fallbackHome, "--exec", "/bin/sh", "-c", "exit 0") | Out-Null
}
finally {
    try {
        Invoke-Captured "wsl.exe" @(
            "--distribution", $distro,
            "--user", "root",
            "--exec", "/bin/sh", "-c",
            "printf '[automount]\nenabled=true\n[user]\ndefault=root\n' > /etc/wsl.conf"
        ) | Out-Null
        Invoke-Captured "wsl.exe" @("--terminate", $distro) | Out-Null
    }
    catch {
        Write-Warning "T062 cleanup could not restore WSL automount: $_"
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
        mapped_cwd_rejected = $true
        mapped_cwd_exit_code = $mismatch.exit_code
        fallback_home = $fallbackHome
        fallback_launch = "PASS"
    }
}

$summary | ConvertTo-Json -Depth 8
