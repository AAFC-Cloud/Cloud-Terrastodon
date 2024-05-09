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
    $params = "--graph-query", "resources | where type =~ 'microsoft.network/publicipaddresses'"
    if (${resp}?.skip_token) {
        $params += "--skip-token", $resp.skip_token
    }
    $resp = az graph query @params | ConvertFrom-Json
    $public_ips += $resp.data
} while ($null -ne $resp.skip_token)

$public_ips `
    | ConvertTo-Json -Depth 100 `
    | Set-Content -Path .\ignore\public_ips.json