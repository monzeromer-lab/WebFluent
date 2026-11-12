# Install the WebFluent compiler, `wf`, from the latest GitHub release.
#
#   irm https://raw.githubusercontent.com/monzeromer-lab/WebFluent/master/install.ps1 | iex
#
# Set $env:WF_INSTALL_DIR to choose where the binary lands, $env:WF_VERSION to
# pin a release ("v5.15.0"). With Rust installed, `cargo install webfluent`
# works too and covers every platform.
$ErrorActionPreference = "Stop"
# Windows PowerShell 5.1 may still offer TLS 1.0 first; GitHub wants 1.2.
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

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
        throw "Could not reach the GitHub API to find the latest version (rate limited?). Set `$env:WF_VERSION = 'v5.15.0'` to pin one."
    }
}
if (-not $Version) { throw "Could not determine the latest version." }
if ($Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+$') { throw "A version is vX.Y.Z, not $Version" }

# Where the release assets come from: GitHub over HTTPS. $env:WF_INSTALL_BASE_URL
# replaces it for testing against a local copy of a release; plain HTTP is
# accepted for a loopback address and nothing else.
if ($env:WF_INSTALL_BASE_URL) {
    $Base = $env:WF_INSTALL_BASE_URL.TrimEnd('/')
    if ($Base -notmatch '^https://' -and $Base -notmatch '^http://(127\.0\.0\.1|localhost)(:[0-9]+)?(/|$)') {
        throw "WF_INSTALL_BASE_URL must be https://, or http:// on 127.0.0.1 or localhost"
    }
    $Base = "$Base/$Version"
} else {
    $Base = "https://github.com/$Repo/releases/download/$Version"
}

$Asset = "wf-$Version-x86_64-windows.zip"
$Url = "$Base/$Asset"
$SumsUrl = "$Base/SHA256SUMS"

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

    # The archive is held to the release's SHA256SUMS before it is unpacked;
    # a missing, malformed or different sum installs nothing.
    $TmpSums = Join-Path $Tmp "SHA256SUMS"
    try {
        Invoke-WebRequest -Uri $SumsUrl -OutFile $TmpSums
    } catch {
        throw "Could not download SHA256SUMS for $Version, so $Asset cannot be checked. Releases before checksums were published can be installed with: cargo install webfluent --version $($Version.TrimStart('v'))"
    }
    $Want = $null
    foreach ($Line in Get-Content -Path $TmpSums) {
        $Parts = $Line.Trim() -split '\s+', 2
        if ($Parts.Count -eq 2 -and $Parts[1].TrimStart('*') -ceq $Asset) { $Want = $Parts[0]; break }
    }
    if (-not $Want) { throw "SHA256SUMS lists no $Asset; refusing to install an archive that cannot be checked." }
    if ($Want -notmatch '^[0-9a-fA-F]{64}$') { throw "SHA256SUMS has a malformed entry for $Asset." }
    $Have = (Get-FileHash -Algorithm SHA256 -Path $TmpZip).Hash
    if ($Have.ToLowerInvariant() -ne $Want.ToLowerInvariant()) {
        throw "$Asset does not match the release's SHA256SUMS (expected $Want, got $Have). Nothing was installed."
    }
    Write-Host "Checked $Asset against SHA256SUMS."

    Expand-Archive -Path $TmpZip -DestinationPath $Tmp -Force

    $Exe = Join-Path $Tmp "wf.exe"
    if (-not (Test-Path $Exe)) { throw "The archive did not contain wf.exe" }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Move-Item -Path $Exe -Destination (Join-Path $InstallDir "wf.exe") -Force
    # Windows already has a `wf`: C:\Windows\System32\WF.msc, the firewall
    # console. System32 comes first on PATH and `.MSC` is in PATHEXT, so a bare
    # `wf` opens the firewall whatever we do to the user's PATH. The same
    # binary is installed as webfluent.exe, which nothing else claims. A
    # release from before it was in the zip is copied here instead.
    Copy-Item -Path (Join-Path $InstallDir "wf.exe") -Destination (Join-Path $InstallDir "webfluent.exe") -Force
} finally {
    Remove-Item -Recurse -Force $Tmp -ErrorAction SilentlyContinue
}

Write-Host "Installed webfluent to $InstallDir\webfluent.exe (and wf.exe)"

# Put it on PATH, once.
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
    Write-Host "Added $InstallDir to your PATH — open a new terminal for it to take effect."
} else {
    Write-Host "$InstallDir is already on your PATH."
}

Write-Host ""
Write-Host "Done! Run 'webfluent --help' to get started."
Write-Host "On Windows a bare 'wf' opens the Windows Firewall console (WF.msc), which"
Write-Host "comes first on PATH. Type 'webfluent' or 'wf.exe' instead; every 'wf ...'"
Write-Host "command in the docs works as 'webfluent ...'."
