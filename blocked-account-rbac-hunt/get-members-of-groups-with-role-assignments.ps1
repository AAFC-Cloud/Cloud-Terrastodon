if ($groups_with_role_assignments.Count -gt $disabled.Count) {
    Write-Warning "There are more groups with role assignments than there are disabled accounts, querying in the opposite direction could be more efficient."
}

New-Item -Path .\ignore\groups -ItemType Directory -ErrorAction SilentlyContinue | Out-Null

$group_members = @()
$scriptBlock = {
    $group_id = $_
    if (Test-Path .\ignore\groups\$group_id.json) {
        $found = Get-Content .\ignore\groups\$group_id.json
        try {
            $found = $found | ConvertFrom-Json
            Write-Host "Found $($found.Count) cached members for group $group_id"
            return $found | ForEach-Object { 
                [PSCustomObject]@{
                    group_id = $group_id
                    member_id = $_.id
                }
            }
        } catch {
            Write-Host "Failed to load $group_id.json! Purging cached file."
            Remove-Item -Path .\ignore\groups\$group_id.json
        }
    }
    $x = az ad group member list --group $group_id
    $x > .\ignore\groups\$group_id.json
    $x = $x | ConvertFrom-Json
    Write-Host "Fetched $($x.Count) members for group $group_id"
    return $x | ForEach-Object { 
        [PSCustomObject]@{
            group_id = $group_id
            member_id = $_.id
        }
    }
}
$results = $groups_with_role_assignments.id `
    | ForEach-Object -Parallel $scriptBlock -ThrottleLimit 20
foreach ($result in $results) {
    $group_members += $result
}
Write-Host "Loaded $($group_members.Count) group member relations!" -ForegroundColor Green