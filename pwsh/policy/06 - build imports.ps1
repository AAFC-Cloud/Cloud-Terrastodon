$assignments = Get-Content .\outputs\intermediate\assignments.json -Raw | ConvertFrom-Json
$definitions = Get-Content .\outputs\intermediate\definitions.json -Raw | ConvertFrom-Json

# We want to build an imports.tf file
$template = @"
import {
    id = "%ID%"
    to = %PROVIDER_TYPE%.%NAME%
}

"@

$result = ""

function sanitize($name) {
    $name -replace "[^\w_]","_"
}
$seen = New-Object 'System.Collections.Generic.HashSet[string]'
$count = 0
foreach ($v in $assignments) {
    $entry = $template
    $entry = $entry -replace "%ID%",$v.id
    $name = $(sanitize $v.displayName)
    if ([string]::IsNullOrWhiteSpace($name)) {
        $name = $(sanitize $v.name)
    }
    if ([string]::IsNullOrWhiteSpace($name)) {
        Write-Warning "Couldn't determine name for assignment with id, skipping $($v.id)"
        continue
    }
    if ($seen.Contains($name)) {
        Write-Warning "Name $name from resource $v already seen! Skipping"
        continue
    }
    $seen.Add($name) | Out-Null
    $entry = $entry -replace "%NAME%",$name
    if ($v.scope -like "/providers/Microsoft.Management/managementGroups/*") {
        $entry = $entry -replace "%PROVIDER_TYPE%","azurerm_management_group_policy_assignment"
    } else {
        Write-Warning "Couldn't determine provider type for scope '$($v.scope)'"
    }
    $result += $entry
    $count += 1
}
Write-Host "Identified $count policy assignments"
$seen.Clear() | Out-Null
$count = 0
foreach ($v in $definitions) {
    if ($v.policyType -ne "Custom") {
        continue;
    }
    $entry = $template
    $entry = $entry -replace "%ID%",$v.id
    $name = $(sanitize $v.displayName)
    if ([string]::IsNullOrWhiteSpace($name)) {
        $name = $(sanitize $v.name)
    }
    if ([string]::IsNullOrWhiteSpace($name)) {
        Write-Warning "Couldn't determine name for definition with id, skipping $($v.id)"
        continue
    }
    if ($seen.Contains($name)) {
        Write-Warning "Name $name from resource $v already seen! Skipping"
        continue
    }
    $seen.Add($name) | Out-Null
    $entry = $entry -replace "%NAME%",$name
    $entry = $entry -replace "%PROVIDER_TYPE%","azurerm_policy_definition"
    $result += $entry
    $count += 1
}
Write-Host "Identified $count policy definitions"
Set-Content .\outputs\intermediate\imports.tf $result