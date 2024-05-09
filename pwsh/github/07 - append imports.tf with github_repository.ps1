$out_file = ".\ignore\terraform\imports.tf"

$content = '
import {
  id = "%REPO%"
  to = github_repository.%NAME%
  provider = github.%ORG%
}'

Clear-Content -Path $out_file -ErrorAction SilentlyContinue

Get-Content .\ignore\orgs.json `
| ConvertFrom-Json `
| ForEach-Object { 
  $org = $_.login
  $repos = gh repo list $org --json name `
  | ConvertFrom-Json `
  | Select-Object -ExpandProperty name
  $repos | ForEach-Object {
    $repo = $_
    $content `
    -replace "%REPO%", $repo `
    -replace "%NAME%", ($repo -replace "[^\w]","_") `
    -replace "%ORG%", $org
  }
} `
| Add-Content -Path $out_file