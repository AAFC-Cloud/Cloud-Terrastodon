$pim_assignments_to_import = $pim_assignments `
    | ForEach-Object { $_.properties.expandedProperties } `
    | Sort-Object -Property { $_ | ConvertTo-Json } -Unique

$entry_template = @"
import {
    id = "%ID%"
    to = azurerm_pim_eligible_role_assignment.%NAME%
}

"@

$content = ""


function sanitize($name) {
    $name -replace "[^a-zA-Z0-9]", "_"
}
foreach ($v in $pim_assignments_to_import) {
    $scope_id = $v.scope.id
    $scope_name = sanitize($v.scope.displayName)

    $role_definition_id = $v.roleDefinition.id
    $role_definition_name = sanitize($v.roleDefinition.displayName)

    $principal_id = $v.principal.id
    $principal_name = sanitize($v.principal.displayName)

    $ID = "$scope_id|$role_definition_id|$principal_id"
    $NAME = "${scope_name}____${role_definition_name}____${principal_name}"

    Add-Member -InputObject $v -MemberType NoteProperty -Name ResourceName -Value $NAME -Force
    $content += $entry_template -replace "%ID%", $ID -replace "%NAME%", $NAME
}
Set-Content ".\ignore\pim_imports.tf" $content
Write-Host "Wrote $($content.Length) chars to pim_imports.tf"
