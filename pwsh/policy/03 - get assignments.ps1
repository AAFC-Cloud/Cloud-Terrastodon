$outfile = ".\outputs\intermediate\assignments.json"
if (Test-Path -Path $outfile) {
    Write-Host "Using cached output: $outfile"
    exit
}

$assignments = @()
$groups = Get-Content .\inputs\management-group-names.txt
foreach ($group in $groups) {
    $assignments += az policy assignment list `
    --scope "/providers/Microsoft.Management/managementGroups/$mgmt" `
    --disable-scope-strict-match `
    | ConvertFrom-Json
}

New-Item -Path outputs -ItemType Directory -ErrorAction SilentlyContinue
New-Item -Path outputs\intermediate -ItemType Directory -ErrorAction SilentlyContinue
$assignments `
| ConvertTo-Json -Depth 100 `
| Set-Content -Path $outfile

$total = $($assignments | ConvertFrom-Json).Count
Write-Host "Found $total policy assignments"