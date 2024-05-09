Write-Host -ForegroundColor Green "Gathering policy assignments..."
$assignments = az policy assignment list --disable-scope-strict-match | ConvertFrom-Json
$choice = $assignments.displayName | fzf
if ([string]::IsNullOrWhiteSpace($choice)) {
    Write-Warning "Bad choice"
    return
}
$policyAssignmentId = $assignments | Where-Object { $_.displayName -eq $choice } | Select-Object -First 1 -ExpandProperty id

Write-Host -ForegroundColor Green "Gathering policy assignment compliance..."
$compliance = az policy state list `
    --filter "policyAssignmentId eq '$policyAssignmentId'" `
    | ConvertFrom-Json


$compliance `
    | Where-Object { $_.complianceState -eq "NonCompliant" } `
    | Group-Object -Property resourceId `
    | ForEach-Object { [PSCustomObject]@{
        resource_id = $_.Name;
        policy_definition_reference_ids = $_.Group.policyDefinitionReferenceId;
    }} `
    | ConvertTo-Json `
    | code -