if (Test-Path .\ignore\users.json) {
    $users_json = Get-Content .\ignore\users.json
    Write-Host "Loaded $($users_json.Length) lines from cached users.json"
} else {
    Write-Host "Fetching users"
    $users_json = az ad user list
    $users_json > .\ignore\users.json
}
$users = $users_json | ConvertFrom-Json
$userz = $users | Group-Object -Property id -AsHashTable
$null = $userz # suppress unused warning
Write-Host "Found $($users.Count) users!" -ForegroundColor Green