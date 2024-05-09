$outfile = ".\outputs\intermediate\definitions.json"
if (Test-Path -Path $outfile) {
    Write-Host "Using cached output: $outfile"
    exit
}
$mgmt = Get-Content -Raw .\inputs\management-group-name.txt
$definitions = az policy definition list `
    --management-group $mgmt

$definitions > $outfile

$definitions = $definitions | ConvertFrom-Json
$total = $definitions.Count
$builtin = $($definitions | Where-Object { $_.policyType -eq "BuiltIn" }).Count
$custom = $total - $builtin
Write-Host "Found $($definitions.Count) policy definitions ($custom custom, $builtin builtin, $total total)"