if (Test-Path .\ignore\public_ips.json) {
    return "public_ips.json already exists";
}
# $subs = az account list | ConvertFrom-Json
# $public_ips = $subs | ForEach-Object -ThrottleLimit 5 -Parallel {
#     $sub = $_
#     Write-Host -ForegroundColor Yellow "Gathering public IPs from subscription $($sub.name)"
#     $ips = az network public-ip list `
#         --subscription $($sub.id) `
#         | ConvertFrom-Json
#     return $ips
# }
$public_ips = @()
do {
    $resp = az graph query `
    --graph-query "resources | where type =~ 'microsoft.network/publicipaddresses'" `
    | ConvertFrom-Json
    $public_ips += $resp.data
} until ($null -eq $resp.skip_token)

$public_ips `
    | ConvertTo-Json -Depth 100 `
    | Set-Content -Path .\ignore\public_ips.json