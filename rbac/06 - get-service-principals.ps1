if (Test-Path .\ignore\service_principals.json) {
    $service_principals_json = Get-Content .\ignore\service_principals.json
    Write-Host "Loaded $($service_principals_json.Length) lines from cached service_principals.json"
} else {
    Write-Host "Fetching service principals"
    $service_principals_json = az ad sp list --all
    $service_principals_json > .\ignore\service_principals.json
}
$service_principals = $service_principals_json | ConvertFrom-Json
$service_principalz = $service_principals | Group-Object -Property id -AsHashTable
$null = $service_principalz # suppress unused warning
Write-Host "Found $($service_principals.Count) service principals!" -ForegroundColor Green