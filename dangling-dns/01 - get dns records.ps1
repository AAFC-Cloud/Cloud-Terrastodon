if (Test-Path .\ignore\dns.json) {
    return "dns.json already exists";
}

$subs = az account list | ConvertFrom-Json
$dns_records = $subs | ForEach-Object -ThrottleLimit 5 -Parallel {
    $sub = $_
    Write-Host -ForegroundColor Yellow "Gathering DNS stuff from subscription $($sub.name)"
    $zones = az network dns zone list `
        --subscription $($sub.id) `
        | ConvertFrom-Json
    $zone_records = $zones | ForEach-Object -ThrottleLimit 5 -Parallel {
        $sub = $using:sub
        $zone = $_
        Write-Host "Gathering DNS records from zone $($sub.name) | $($zone.resourceGroup) | $($zone.name)"
        $records = az network dns record-set list `
            --resource-group $zone.resourceGroup `
            --zone-name $zone.name `
            --subscription $($sub.id) `
            | ConvertFrom-Json
        [PSCustomObject]@{
            Zone = $zone
            Records = $records
        }
    }
    return [PSCustomObject]@{
        sub = $sub
        zones = $zone_records ?? @()
    }
}
$dns_records | ConvertTo-Json -Depth 100 | Set-Content -Path .\ignore\dns.json