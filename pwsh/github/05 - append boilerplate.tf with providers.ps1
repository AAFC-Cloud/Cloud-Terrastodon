$out_file = ".\ignore\terraform\boilerplate.tf"

$content = '
provider "github" {
  owner = "%NAME%"
  alias = "%NAME%"
}'

Get-Content .\ignore\orgs.json `
| ConvertFrom-Json `
| Select-Object -ExpandProperty login `
| ForEach-Object { $content -replace "%NAME%", $_ }
| Add-Content -Path $out_file