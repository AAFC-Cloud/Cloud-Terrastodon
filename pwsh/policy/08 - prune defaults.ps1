$content = Get-Content .\outputs\intermediate\generated.tf
# metadata             =*
$prune = @"
# __generated__ by Terraform*
# Please review these resources and move them into your main configuration files.
  enforce              = true
  description          = null
  not_scopes           = []
  description         = null
  parameters          = null
    identity_ids = []
"@ -replace "`r","" -split "`n"
$prunePatterns = $prune | Where-Object { $_.Contains("*") }
$new_content = $content | Where-Object {
    $line = $_
    if ($prune -contains $line) {
        return $false
    }
    $matched = $prunePatterns | Where-Object { $line -like $_ }
    return $matched.Count -eq 0
}
Set-Content -Value $new_content -Path .\outputs\intermediate\generated-pruned.tf


$contentLength = $content.Length
$newContentLength = $new_content.Length

# Calculate the reduction in size
$reduction = $contentLength - $newContentLength

# Calculate the percentage reduction and round it to the nearest whole number
$percentageReduction = [Math]::Round(($reduction / $contentLength) * 100)

# Display the result using Write-Host
Write-Host "The content was reduced by $percentageReduction%."

