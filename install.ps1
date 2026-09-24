# Install the WebFluent compiler, `wf`, from the latest GitHub release.
#
#   irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex
#
# Set $env:WF_INSTALL_DIR to choose where the binary lands, $env:WF_VERSION to
# pin a release ("v3.2.1"). With Rust installed, `cargo install webfluent`
# works too and covers every platform.
$ErrorActionPreference = "Stop"

$Repo = "monzeromer-lab/WebFluent"
$InstallDir = if ($env:WF_INSTALL_DIR) { $env:WF_INSTALL_DIR } else { "$env:USERPROFILE\.webfluent\bin" }

# The release publishes `wf-<version>-x86_64-windows.zip`. This used to ask for
# a Rust target triple — `x86_64-pc-windows-msvc` — which names no asset that
# exists, so the download was a 404 on every run.
$Version = if ($env:WF_VERSION) {
    $env:WF_VERSION
} else {
    try {
        (Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest").tag_name
    } catch {
        throw "Could not reach the GitHub API to find the latest version (rate limited?). Set `$env:WF_VERSION = 'v3.2.1'` to pin one."
    }
}
if (-not $Version) { throw "Could not determine the latest version." }

$Asset = "wf-$Version-x86_64-windows.zip"
$Url = "https://github.com/$Repo/releases/download/$Version/$Asset"

Write-Host "Installing wf $Version (x86_64-windows)..."

# Unpack through a temporary directory so a failed download cannot leave a
# half-written binary where a working one used to be.
$Tmp = Join-Path $env:TEMP ("wf-" + [System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null
try {
    $TmpZip = Join-Path $Tmp "wf.zip"
    try {
        Invoke-WebRequest -Uri $Url -OutFile $TmpZip
    } catch {
        throw "Could not download $Asset. Check https://github.com/$Repo/releases for what that release carries."
    }
    Expand-Archive -Path $TmpZip -DestinationPath $Tmp -Force

    $Exe = Join-Path $Tmp "wf.exe"
    if (-not (Test-Path $Exe)) { throw "The archive did not contain wf.exe" }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Move-Item -Path $Exe -Destination (Join-Path $InstallDir "wf.exe") -Force
} finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}

Write-Host "Installed wf to $InstallDir\wf.exe"

# Put it on PATH, once.
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
    Write-Host "Added $InstallDir to your PATH — open a new terminal for it to take effect."
} else {
    Write-Host "$InstallDir is already on your PATH."
}

Write-Host "Done! Run 'wf --help' to get started."
