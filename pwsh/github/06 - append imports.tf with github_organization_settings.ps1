$out_file = ".\ignore\terraform\imports.tf"

$content = '
import {
  id = "%ID%"
  to = github_organization_settings.%NAME%
  provider = github.%NAME%
}'

Clear-Content -Path $out_file -ErrorAction SilentlyContinue

Get-Content .\ignore\orgs.json `
| ConvertFrom-Json `
| ForEach-Object { 
    $content `
    -replace "%NAME%", $_.login `
    -replace "%ID%", $_.id
} `
| Add-Content -Path $out_file