# Switches the active Pogly layout to match an OBS scene.
# Wire to Streamer.bot's "OBS Scene Changed" event:
#   powershell -ExecutionPolicy Bypass -File sync-layout.ps1 -Scene "%obsSceneName%"
# Scenes without a matching Pogly layout are skipped quietly, so it is safe
# to fire on every scene change.
param(
    [Parameter(Mandatory = $true)][string]$Scene
)

pogly layouts set-active --name $Scene 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "No Pogly layout named '$Scene' - skipped."
    exit 0
}
