$outfile = ".\outputs\intermediate\assignments.json"
if (Test-Path -Path $outfile) {
    Write-Host "Using cached output: $outfile"
    exit
}

$mgmt = Get-Content -Raw .\inputs\management-group-name.txt
$assignments = az policy assignment list `
    --scope "/providers/Microsoft.Management/managementGroups/$mgmt" `
    --disable-scope-strict-match

New-Item -Path outputs -ItemType Directory -ErrorAction SilentlyContinue
New-Item -Path outputs\intermediate -ItemType Directory -ErrorAction SilentlyContinue
$assignments > $outfile

$total = $($assignments | ConvertFrom-Json).Count
Write-Host "Found $total policy assignments"