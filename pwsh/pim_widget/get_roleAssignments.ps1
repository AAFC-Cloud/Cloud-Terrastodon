Write-Host "Getting your object id"
$object_id = az ad signed-in-user show --query "id" -o tsv
Write-Host "Patching URL"
$content = Get-Content .\url_roleAssignments.txt
$content = $content -replace "~~ID HERE~~",$object_id
Set-Content -Path .\ignore\url_roleAssignments.txt -Value $content
Write-Host "Fetching"
az rest --method GET --url '@ignore/url_roleAssignments.txt' --resource "https://portal.azure.com"