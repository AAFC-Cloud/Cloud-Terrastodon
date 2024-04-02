if (Test-Path .\ignore\subscriptions.json) {
    $subscriptions = Get-Content .\ignore\subscriptions.json
    Write-Host "Loaded $($subscriptions.Length) lines from subscriptions.json cache"
} else {
    # --no-register <= https://github.com/Azure/azure-cli/issues/19511
    $subscriptions = az account list
    $subscriptions > .\ignore\subscriptions.json
    Write-Host "Fetched $($subscriptions.Length) lines into subscriptions.json"
}
$subscriptions = $subscriptions | ConvertFrom-Json
$subscriptionz = $subscriptions | Group-Object -Property id -AsHashTable
$null = $subscriptionz # suppress unused warning
Write-Host "Found $($subscriptions.Count) subscriptions!" -ForegroundColor Green