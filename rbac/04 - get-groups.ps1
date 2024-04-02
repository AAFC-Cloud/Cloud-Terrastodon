if (Test-Path .\ignore\groups.json) {
    $groups = Get-Content .\ignore\groups.json
    Write-Host "Loaded $($groups.Length) lines from groups.json cache"
} else {
    $groups = az ad group list
    $groups > .\ignore\groups.json
    Write-Host "Fetched $($groups.Length) lines into groups.json"
}
$groups = $groups | ConvertFrom-Json
$groupz = $groups | Group-Object -Property id -AsHashTable
$null = $groupz # suppress unused warning
Write-Host "Found $($groups.Count) groups!" -ForegroundColor Green