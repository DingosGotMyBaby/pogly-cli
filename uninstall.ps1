# Uninstaller for pogly-cli.
# Removes the binaries, version store, and PATH entry. Overlay profiles are
# kept unless you pass -PurgeConfig.
param(
    [switch]$PurgeConfig
)
$ErrorActionPreference = "Stop"

$root = Join-Path ([Environment]::GetFolderPath("LocalApplicationData")) "Pogly\cli"
$configDir = Join-Path ([Environment]::GetFolderPath("ApplicationData")) "Pogly\cli"

if (Test-Path $root) {
    Remove-Item -Recurse -Force $root
    Write-Host "Removed $root"
} else {
    Write-Host "Nothing installed at $root"
}

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$cleaned = ($userPath -split ";") | Where-Object { $_ -and $_ -ne $root }
if (($cleaned -join ";") -ne $userPath) {
    [Environment]::SetEnvironmentVariable("Path", ($cleaned -join ";"), "User")
    Write-Host "Removed $root from your user PATH"
}

if ($PurgeConfig) {
    if (Test-Path $configDir) {
        Remove-Item -Recurse -Force $configDir
        Write-Host "Removed overlay profiles at $configDir"
    }
} elseif (Test-Path $configDir) {
    Write-Host "Overlay profiles kept at $configDir (rerun with -PurgeConfig to remove them)"
}

Write-Host "pogly-cli uninstalled."
