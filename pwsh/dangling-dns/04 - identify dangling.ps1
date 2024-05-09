$dns = Get-Content -Raw ".\ignore\dns.json" | ConvertFrom-Json -Depth 25
$resources = Get-Content -Raw ".\ignore\resource_endpoints.json" | ConvertFrom-Json -Depth 25
$ips = Get-Content -Raw ".\ignore\public_ips.json" | ConvertFrom-Json 


$ipv4_list = $dns.zones.records.ARecords.ipv4Address
$fqdn_list = $dns.zones.records.CNAMERecord.cname

# We want to identify list entries that don't have a corresponding azure resource
$fqdn_not_found = $fqdn_list `
| Where-Object { $resources.dnsEndpoints -notcontains $_ }

$ipv4_not_found = $ipv4_list `
| Where-Object { $ips.properties.ipAddress -notcontains $_ }

if ($fqdn_not_found.Count -gt 0) {
    Write-Warning "Found CNAME records that don't have a corresponding Azure resource"
}

if ($ipv4_not_found.Count -gt 0) {
    Write-Warning "Found A records that don't have a corresponding Azure resource"
}

$bad_cname_records = $dns.zones.records `
| Where-Object { $fqdn_not_found -contains $_.CNAMERecord.cname }

$bad_a_records = $dns.zones.records `
| Where-Object {
    (
        $_.ARecords.ipv4Address `
        | Where-Object { $ipv4_not_found -contains $_ } `
        | Measure-Object
    ).Count -ne 0
}

$bad_records = @($bad_cname_records + $bad_a_records) `
    | ForEach-Object { $_.type + " " + $_.fqdn }

$zone_bad_measure = $dns.zones `
    | ForEach-Object { 
        $bad_record_count = (
            $_.records `
                | Where-Object { 
                    $bad_cname_records -contains $_ `
                    -or $bad_a_records -contains $_
                }
        ).Count
        $total_record_count = $_.records.Count
        "{0,-40} {1,3} of {2,3} records are sus ({3,3:N0}%)" -f $_.Zone.name, $bad_record_count, $total_record_count, ($bad_record_count/$total_record_count*100)
    }

[PSCustomObject]@{
    summary = @{
        fqdn_not_found = $fqdn_not_found
        ipv4_not_found = $ipv4_not_found
        bad_records = $bad_records
        zone_summary = $zone_bad_measure
    }
    bad_cname_records = $bad_cname_records
    bad_a_records = $bad_a_records
} `
    | ConvertTo-Json -Depth 100 `
    | Set-Content .\ignore\output.json
