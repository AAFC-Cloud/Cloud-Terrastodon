$groups_to_import = $role_assignments_to_import `
    | Where-Object { $_.principalType -eq "Group" } `
    | ForEach-Object { $_.principalId } `
    | Sort-Object -Unique `
    | Where-Object { $groupz.ContainsKey($_) } `
    | ForEach-Object { $groupz[$_] }

$groups_to_import += $groups | Where-Object { $_.displayName.StartsWith("AAD-CloudOps") -and -not ($groups_to_import -contains $_) }

$entry_template = @"
import {
    id = "%ID%"
    to = azuread_group.%NAME%
}

"@
$content = ""



function sanitize($name) {
    $name -replace "[^a-zA-Z0-9]", "_"
}
foreach ($g in $groups_to_import) {
    $ID = $g.id
    $NAME = sanitize($g.displayName)
    Add-Member -InputObject $g -MemberType NoteProperty -Name ResourceName -Value $NAME -Force
    $content += $entry_template -replace "%ID%", $ID -replace "%NAME%", $(sanitize $NAME)
}
Set-Content ".\ignore\group_imports.tf" $content
Write-Host "Wrote $($content.Length) chars to group_imports.tf"