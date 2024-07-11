Write-Host "Getting your object id"
$object_id = az ad signed-in-user show --query "id" -o tsv

Write-Host "Patching URL"
$content = @'
https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$expand=linkedEligibleRoleAssignment,subject,roleDefinition($expand=resource)&$filter=(subject/id%20eq%20%27~~ID HERE~~%27)+and+(assignmentState%20eq%20%27Eligible%27)&$count=true
'@
$content = $content -replace "~~ID HERE~~",$object_id
New-Item -ItemType Directory -Path ignore -ErrorAction SilentlyContinue | Out-Null
Set-Content -Path .\ignore\url_roleAssignments.txt -Value $content

Write-Host "Fetching"
az rest --method GET --url '@ignore/url_roleAssignments.txt'

# This github issue details some of the problems involved in Entra PIM activation
# https://github.com/Azure/azure-cli/issues/28854
#
# The Microsoft-controlled app registration is missing some scopes to hit some of the endpoints
#
# GET https://api.azrbac.mspim.azure.com/api/v2/privilegedAccess/aadroles/roleAssignments?$expand=linkedEligibleRoleAssignment,subject,scopedResource,roleDefinition($expand=resource)&$filter=(subject/id%20eq%20%27~~ID HERE~~%27)+and+(assignmentState%20eq%20%27Eligible%27)&$count=true
# GET https://graph.microsoft.com/v1.0/identityGovernance/privilegedAccess/group/eligibilitySchedules?`$filter=PrincipalId eq '~~ID HERE~~'
# az account get-access-token --resource-type ms-graph --scope "PrivilegedEligibilitySchedule.Read.AzureADGroup PrivilegedEligibilitySchedule.ReadWrite.AzureADGroup PrivilegedAccess.Read.AzureADGroup PrivilegedAccess.ReadWrite.AzureADGroup"
# 
# This one works tho
# 
# https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$expand=linkedEligibleRoleAssignment,subject,roleDefinition($expand=resource)&$filter=(subject/id%20eq%20%27~~ID HERE~~%27)+and+(assignmentState%20eq%20%27Eligible%27)&$count=true