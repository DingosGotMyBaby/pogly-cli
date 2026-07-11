# Sets the content of an existing text element. Composes with anything that
# tracks a number or string: donation goals, death counters, sub counts.
#   powershell -ExecutionPolicy Bypass -File set-text.ps1 -Id 12 -Text "Subs today: 14"
param(
    [Parameter(Mandatory = $true)][int]$Id,
    [Parameter(Mandatory = $true)][string]$Text
)
$ErrorActionPreference = "Stop"

pogly elements update $Id --text $Text | Out-Null
Write-Host "Element $Id -> '$Text'"
