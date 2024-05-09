$outfile = ".\outputs\intermediate\initiatives.json"
if (Test-Path -Path $outfile) {
    Write-Host "Using cached output: $outfile"
    exit
}
$mgmt = Get-Content -Raw .\inputs\management-group-name.txt
$initiatives = az policy set-definition list `
    --management-group $mgmt

$initiatives > $outfile

$initiatives = $initiatives | ConvertFrom-Json
$total = $initiatives.Count
$builtin = $($initiatives | Where-Object { $_.policyType -eq "BuiltIn" }).Count
$custom = $total - $builtin
Write-Host "Found $($initiatives.Count) policy initiatives ($custom custom, $builtin builtin, $total total)"