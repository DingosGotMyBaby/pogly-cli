# Quick installer for pogly-cli.
#   iwr https://cli.pogly.gg -useb | iex
$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = "PoglyApp/pogly-cli"
$root = Join-Path ([Environment]::GetFolderPath("LocalApplicationData")) "Pogly\cli"

Write-Host "Installing pogly-cli..."

$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" -Headers @{ "User-Agent" = "pogly-cli-installer" } -UseBasicParsing
$tag = $release.tag_name
$version = $tag.TrimStart("v")
$binDir = Join-Path $root "bin\$version"

New-Item -ItemType Directory -Force $binDir | Out-Null

Write-Host "Downloading pogly-cli $tag..."
Invoke-WebRequest -Uri "https://github.com/$repo/releases/download/$tag/pogly-cli.exe" -OutFile (Join-Path $binDir "pogly-cli.exe") -UseBasicParsing

$launcher = Join-Path $root "pogly.exe"
try {
    Invoke-WebRequest -Uri "https://github.com/$repo/releases/download/$tag/pogly.exe" -OutFile $launcher -UseBasicParsing
} catch {
    # A running launcher can't be overwritten, but it can be renamed aside.
    Move-Item $launcher "$launcher.old" -Force
    Invoke-WebRequest -Uri "https://github.com/$repo/releases/download/$tag/pogly.exe" -OutFile $launcher -UseBasicParsing
}

Set-Content -Path (Join-Path $root "version") -Value $version -Encoding ascii -NoNewline

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($userPath -split ";") -notcontains $root) {
    [Environment]::SetEnvironmentVariable("Path", ($userPath.TrimEnd(";") + ";" + $root), "User")
    $env:Path += ";" + $root
    Write-Host "Added $root to your user PATH (open a new terminal if 'pogly' is not found)."
}

Write-Host ""
Write-Host "pogly-cli $tag installed."
Write-Host "Get started:"
Write-Host "  pogly overlay add <overlay-url-or-identity> --token pgly_..."
Write-Host "  pogly help"
