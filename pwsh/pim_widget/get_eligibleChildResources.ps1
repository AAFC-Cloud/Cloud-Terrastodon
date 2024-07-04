Write-Host "Patching URL"
$content = Get-Content .\url_eligibleChildResources.txt
$content = $content -replace "{scope}","idk"
Set-Content -Path .\ignore\url_eligibleChildResources.txt -Value $content
az rest --method GET --url '@ignore/url_eligibleChildResources.txt'