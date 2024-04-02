$groups_json = Get-Content .\outputs\intermediate\groups.json | ConvertFrom-Json
$group_names = Get-Content .\inputs\group_names.txt

$group_lookup_by_name = @{}
Write-Host "Building lookup table"
foreach ($group in $groups_json) {
    if ($group_names.Contains($group.displayName)) {
        $group_lookup_by_name[$group.displayName] = $group
    }
}
Write-Host "We have $($group_lookup_by_name.Keys.Count) of $($group_names.Count) entries"
$content = $group_lookup_by_name | ConvertTo-Json -Depth 5
Set-Content -Value $content -Path .\outputs\intermediate\lookup.json