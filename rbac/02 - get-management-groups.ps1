if (Test-Path .\ignore\management-groups.json) {
    $management_groups = Get-Content .\ignore\management-groups.json
    Write-Host "Loaded $($management_groups.Length) lines from management-groups.json cache"
} else {
    # --no-register <= https://github.com/Azure/azure-cli/issues/19511
    $management_groups = az account management-group list --no-register
    $management_groups > .\ignore\management-groups.json
    Write-Host "Fetched $($management_groups.Length) lines into management-groups.json"
}
$management_groups = $management_groups | ConvertFrom-Json
$management_groupz = $management_groups | Group-Object -Property id -AsHashTable
$null = $management_groupz # suppress unused warning
Write-Host "Found $($management_groups.Count) management groups!" -ForegroundColor Green