if (Test-Path .\ignore\role_assignments.json) {
    $role_assignments = Get-Content .\ignore\role_assignments.json
    Write-Host "Loaded $($role_assignments.Length) lines from role_assignments.json cache"
    $role_assignments = $role_assignments | ConvertFrom-Json
} else {
    Write-Host "Gathering role assignments 🍩" -ForegroundColor Cyan
    $subscriptions = az account list
    $subscriptions = $subscriptions | ConvertFrom-Json
    $role_assignments = @()
    $scriptBlock = {
        $sub = $_
        Write-Host "Fetching role assignments from $($sub.name)"
        $tempAssignments = @()
        $found = az role assignment list --subscription $sub.id --all | ConvertFrom-Json
        foreach ($item in $found) {
            $tempAssignments += $item
        }
        $tempAssignments
    }
    $results = $subscriptions | ForEach-Object -Parallel $scriptBlock -ThrottleLimit 20
    foreach ($result in $results) {
        $role_assignments += $result
    }
    $role_assignments | ConvertTo-Json -Depth 100 > .\ignore\role_assignments.json
    Write-Host "Finished gathering role assignments!"
}
$role_assignments = $role_assignments | Sort-Object -Unique -Property id
Write-Host "Loaded $($role_assignments.Count) role assignments!" -ForegroundColor Green