Write-Host "Getting your object id"
$object_id = az ad signed-in-user show --query "id" -o tsv

Write-Host "Patching URL"
$url = @'
https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$expand=linkedEligibleRoleAssignment,subject,roleDefinition($expand=resource)&$filter=(subject/id%20eq%20%27~~ID HERE~~%27)+and+(assignmentState%20eq%20%27Eligible%27)&$count=true
'@
$url = $url -replace "~~ID HERE~~",$object_id
New-Item -ItemType Directory -Path ignore -ErrorAction SilentlyContinue | Out-Null
Set-Content -Path .\ignore\url_roleAssignments.txt -Value $url

Write-Host "Fetching role assignments"
$roleAssignments = az rest --method GET --url '@ignore/url_roleAssignments.txt' `
| ConvertFrom-Json `
| Select-Object -ExpandProperty value

$chosenRoleAssignment = $roleAssignments `
| ForEach-Object { $_.roleDefinition.displayName } `
| fzf
$chosenRoleAssignment = $roleAssignments `
| Where-Object { $_.roleDefinition.displayName -eq $chosenRoleAssignment } `
| Select-Object -First 1

$reason = Read-Host -Prompt "Reason"

foreach ($ass in $chosenRoleAssignment) {
    Write-Host "Activating $($chosenRoleAssignment.roleDefinition.displayName)"

    $url = @"
    https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignmentRequests
"@.Trim()

    $bodyPath = "ignore/activate_entra_role_body.json"
    $body = [PSCustomObject]@{
        roleDefinitionId= $ass.roleDefinition.id
        resourceId= $ass.resourceId
        subjectId= $ass.subject.id
        assignmentState= "Active"
        type= "UserAdd"
        reason= $reason
        ticketNumber= ""
        ticketSystem= ""
        schedule= [PSCustomObject]@{
            type= "Once"
            startDateTime= $null
            endDateTime= $null
            duration= "PT30M"
        }
        linkedEligibleRoleAssignmentId= $ass.id
        scopedResourceId= ""
    }
    $body | ConvertTo-Json -Depth 100 | Set-Content $bodyPath
    az rest --method POST --url $url --body "@$bodyPath"
}