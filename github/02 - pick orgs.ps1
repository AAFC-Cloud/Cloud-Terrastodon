$out_file = ".\ignore\orgs.json"
if (Test-Path $out_file) {
    Write-Warning "$out_file already exists, skipping"
    return
}

$orgs = gh org list
$choices = $orgs `
| Where-Object { $_ -notlike "Showing * of * organizations" } `
| Where-Object { $_ }

$orgs_to_import = $choices `
| fzf `
    --multi `
    --prompt "Pick orgs to import >"

Write-Host "Fetching info for $($orgs_to_import.Count) orgs"

$data = @()
foreach ($org in $orgs_to_import) {
    Write-Host "Fetching $org"
    $data += gh api "orgs/$org" | ConvertFrom-Json
}

$data `
| ConvertTo-Json `
| Set-Content -Path $out_file
