if (Test-Path .\ignore\users.json) {
    $users_json = Get-Content .\ignore\users.json
    Write-Host "Loaded $($users_json.Length) lines from cached users.json"
} else {
    Write-Host "Fetching users"
    $users_json = az ad user list
    $users_json > .\ignore\users.json
}
$users = $users_json | ConvertFrom-Json

if (Test-Path .\ignore\disabled_user_ids.txt) {
    $disabled_txt = Get-Content .\ignore\disabled_user_ids.txt
    Write-Host "Loaded $($disabled_txt.Length) lines from cached disabled_user_ids.txt"
} else {
    Write-Host "Fetching disabled users"
    $disabled_txt = az ad user list --filter "accountEnabled eq false" --query "[].id" -o tsv
    $disabled_txt > .\ignore\disabled_user_ids.txt
}
$disabled = New-Object 'System.Collections.Generic.HashSet[string]'
foreach ($id in $disabled_txt) {
    $null = $disabled.Add($id)
}

$userz = $users | Group-Object -Property id -AsHashTable
$null = $userz # suppress unused warning
Write-Host "Found $($users.Count) users ($($disabled.Count) disabled)!" -ForegroundColor Green