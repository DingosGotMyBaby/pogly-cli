# Spawns a temporary text alert on the overlay, fades it out, and deletes it.
# Wire this to Streamer.bot / Firebot events:
#   powershell -ExecutionPolicy Bypass -File alert.ps1 -Message "%userName% just subscribed!"
param(
    [Parameter(Mandatory = $true)][string]$Message,
    [int]$Duration = 8,
    [int]$Size = 72,
    [string]$Color = "#82a5ff",
    [int]$X = 760,
    [int]$Y = 100
)
$ErrorActionPreference = "Stop"

$created = pogly --json elements add text --text $Message --size $Size --color $Color --x $X --y $Y | ConvertFrom-Json
$id = $created.element.id

Start-Sleep -Seconds $Duration

foreach ($t in 80, 60, 40, 20) {
    pogly elements update $id --transparency $t | Out-Null
    Start-Sleep -Milliseconds 150
}
pogly elements delete $id | Out-Null
