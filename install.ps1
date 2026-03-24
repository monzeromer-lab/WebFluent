$ErrorActionPreference = "Stop"

$Repo = "monzeromer-lab/WebFluent"
$InstallDir = if ($env:WF_INSTALL_DIR) { $env:WF_INSTALL_DIR } else { "$env:USERPROFILE\.webfluent\bin" }
$Target = "x86_64-pc-windows-msvc"

$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Version = $Release.tag_name

$Url = "https://github.com/$Repo/releases/download/$Version/wf-$Version-$Target.zip"
$TmpZip = "$env:TEMP\wf.zip"

Write-Host "Installing wf $Version..."

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Invoke-WebRequest -Uri $Url -OutFile $TmpZip
Expand-Archive -Path $TmpZip -DestinationPath $InstallDir -Force
Remove-Item $TmpZip

# Add to PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$UserPath", "User")
    Write-Host "Added $InstallDir to user PATH (restart terminal to take effect)"
}

Write-Host "Done! Run 'wf --help' to get started."
