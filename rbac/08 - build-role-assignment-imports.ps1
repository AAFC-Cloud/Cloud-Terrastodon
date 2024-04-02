$role_assignments_to_import = $role_assignments `
| Where-Object { $_.scope -match "^(/providers/Microsoft.Management/managementGroups/[\w-]+|/subscriptions/[\w-]+)`$" } `
    <# deduplicate #> `
| Group-Object -Property id `
| ForEach-Object { $_.Group[0] } 


$entry_template = @"
import {
    id = "%ID%"
    to = azurerm_role_assignment.%NAME%
}

"@
$content = ""



function sanitize($name) {
    $name -replace "[^a-zA-Z0-9]", "_"
}
foreach ($ra in $role_assignments_to_import) {
    $ID = $ra.id
    switch -Regex ($ra.scope) {
        "^/providers/Microsoft.Management/managementGroups/[\w-]+`$" {
            $scope = "MGMT__$($management_groupz[$ra.scope].name)"
            break
        }

        "^/subscriptions/([\w-]+)`$" {
            $sub_id = $matches[1]
            $scope = "SUB__$($subscriptionz[$sub_id].name)"
            break
        }
        Default {
            Write-Warning "Bad scope: $($ra.scope)"
            exit 1
        }
    }
    switch ($ra.principalType) {
        "User" {
            $principal_name = $userz[$ra.principalId].displayName
            break
        }
        "Group" {
            $principal_name = $groupz[$ra.principalId].displayName
        }
        "ServicePrincipal" {
            if ($service_principalz.ContainsKey($ra.principalId)) {
                $sp = $service_principalz[$ra.principalId]
                if ($sp.servicePrincipalType -eq "ManagedIdentity" -and $sp.alternativeNames.Count -ge 2 -and $sp.alternativeNames[1] -like "*microsoft.authorization/policyAssignments*") {
                    $principal_name = "POLICY__$($sp.displayName)"
                } else {
                    $principal_name = $sp.displayName
                }
            }
            else {
                $randomHex = -join ((48..57) + (97..102) | Get-Random -Count 8 | % { [char]$_ })
                $principal_name = "UNKNOWN__$randomHex"
            }
        }
        Default {
            Write-Warning "Bad principal type: $(ra.principalType)"
            exit 1
        }
    }
    $NAME = "${scope}____$($ra.roleDefinitionName)____$($ra.principalType.ToUpper())__$principal_name"
    $content += $entry_template -replace "%ID%", $ID -replace "%NAME%", $(sanitize $NAME)
}
Set-Content ".\ignore\role_assignment_imports.tf" $content
Write-Host "Wrote $($content.Length) chars to role_assignment_imports.tf"