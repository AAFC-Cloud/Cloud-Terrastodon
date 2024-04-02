$disabled_group_members = $group_members | Where-Object { $_.member_id -in $disabled }
$disabled_group_memberz = $disabled_group_members | Group-Object -Property group_id -AsHashTable
if ($null -eq $disabled_group_memberz) {
    $disabled_group_memberz = @{}
}

$results = ""
$results += "# Role Assignment Against Disabled User`n`n"

foreach ($ra in $role_assignments) {
    if ($ra.principalId -in $disabled) {
        # Role assignment targets disabled user
        
        $results += "## Role Assignment`n`n"
        $results += "``````json`n"
        $results += $ra | ConvertTo-Json
        $results += "`n```````n`n"
        
        $user = $userz[$ra.principalId]
        $results += "### User`n`n"
        $results += "``````json`n"
        $results += $user | ConvertTo-Json
        $results += "`n```````n`n"

        $results += "`n`n---`n---`n---`n`n"
    } elseif ($disabled_group_memberz.ContainsKey($ra.principalId)) {
        # Role assignment targets group containing disabled user

        $results += "## Role Assignment`n`n"
        $results += "``````json`n"
        $results += $ra | ConvertTo-Json
        $results += "`n```````n`n"

        $group = $groupz[$ra.principalId]
        $results += "### Group`n`n"
        $results += "``````json`n"
        $results += $group | ConvertTo-Json
        $results += "`n```````n`n"

        $results += "#### Disabled Group Members`n`n"
        foreach ($membership in $disabled_group_memberz[$ra.principalId]) {
            $user = $userz[$membership.member_id]
            $results += "``````json`n"
            $results += $user | ConvertTo-Json
            $results += "`n```````n`n"
            $results += "`n`n---`n`n"
        }
        
        $results += "`n`n---`n---`n---`n`n"
    }
}

$results += "# Group Memberships Against Disabled Users`n`n"
foreach ($membership in $disabled_group_members) {
    $group = $groupz[$membership.group_id]
    $results += "## Group`n`n"
    $results += "``````json`n"
    $results += $group | ConvertTo-Json
    $results += "`n```````n`n"

    $user = $userz[$membership.member_id]
    $results += "### User`n`n"
    $results += "``````json`n"
    $results += $user | ConvertTo-Json
    $results += "`n```````n`n"
    $results += "`n`n---`n---`n---`n`n"
}

Set-Content .\ignore\output.md $results
Write-Host -ForegroundColor Green "Wrote $($results.Length) chars to output.md!"