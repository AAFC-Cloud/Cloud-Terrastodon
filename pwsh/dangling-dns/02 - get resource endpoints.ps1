if (Test-Path .\ignore\resource_endpoints.json) {
    return "resource_endpoints.json already exists";
}

# https://github.com/Azure/Azure-Network-Security/blob/4933168563a3686ff8d7866ef3dda66c4254ffda/Cross%20Product/DNS%20-%20Find%20Dangling%20DNS%20Records/AzDanglingDomain/Export/Get-DanglingDnsRecords.ps1#L823-L884
$query = @"
resources
    | where type in ('microsoft.network/frontdoors',
    'microsoft.storage/storageaccounts',
    'microsoft.cdn/profiles/endpoints',
    'microsoft.cdn/profiles/afdendpoints',
    'microsoft.network/publicipaddresses',
    'microsoft.network/trafficmanagerprofiles',
    'microsoft.containerinstance/containergroups',
    'microsoft.apimanagement/service',
    'microsoft.web/sites',
    'microsoft.web/sites/slots',
    'microsoft.classiccompute/domainnames',    
    'microsoft.classicstorage/storageaccounts')
    |mvexpand properties.hostnameConfigurations    
    | extend dnsEndpoint = case
    (
       type =~ 'microsoft.network/frontdoors', properties.cName,
       type =~ 'microsoft.storage/storageaccounts', iff(properties['primaryEndpoints']['blob'] matches regex '(?i)(http|https)://',
                parse_url(tostring(properties['primaryEndpoints']['blob'])).Host, tostring(properties['primaryEndpoints']['blob'])),
       type =~ 'microsoft.cdn/profiles/endpoints', properties.hostName,
       type =~ 'microsoft.cdn/profiles/afdendpoints', properties.hostName,
       type =~ 'microsoft.network/publicipaddresses', properties.dnsSettings.fqdn,
       type =~ 'microsoft.network/trafficmanagerprofiles', properties.dnsConfig.fqdn,
       type =~ 'microsoft.containerinstance/containergroups', properties.ipAddress.fqdn,
       type =~ 'microsoft.apimanagement/service', properties_hostnameConfigurations.hostName,
       type =~ 'microsoft.web/sites', properties.defaultHostName,
       type =~ 'microsoft.web/sites/slots', properties.defaultHostName,
       type =~ 'microsoft.classiccompute/domainnames',properties.hostName,
       ''
    )
    | extend dnsEndpoints = case
    (
        type =~ 'microsoft.apimanagement/service', 
           pack_array(dnsEndpoint, 
            parse_url(tostring(properties.gatewayRegionalUrl)).Host,
            parse_url(tostring(properties.developerPortalUrl)).Host, 
            parse_url(tostring(properties.managementApiUrl)).Host,
            parse_url(tostring(properties.portalUrl)).Host,
            parse_url(tostring(properties.scmUrl)).Host,
            parse_url(tostring(properties.gatewayUrl)).Host),
        type =~ 'microsoft.web/sites', properties.hostNames,
       	type =~ 'microsoft.web/sites/slots', properties.hostNames,
        type =~ 'microsoft.classicstorage/storageaccounts', properties.endpoints,
        pack_array(dnsEndpoint)
    )
    | where isnotempty(dnsEndpoint)
    | extend resourceProvider = case
    (
        dnsEndpoint endswith 'azure-api.net', 'azure-api.net',
        dnsEndpoint endswith 'azurecontainer.io', 'azurecontainer.io',
        dnsEndpoint endswith 'azureedge.net', 'azureedge.net',
        dnsEndpoint endswith 'azurefd.net', 'azurefd.net',
        dnsEndpoint endswith 'azurewebsites.net', 'azurewebsites.net',
        dnsEndpoint endswith 'blob.core.windows.net', 'blob.core.windows.net', 
        dnsEndpoint endswith 'cloudapp.azure.com', 'cloudapp.azure.com',
        dnsEndpoint endswith 'cloudapp.net', 'cloudapp.net',
        dnsEndpoint endswith 'trafficmanager.net', 'trafficmanager.net',
        '' 
    )
    | project id, tenantId, subscriptionId, type, resourceGroup, name, dnsEndpoint, dnsEndpoints, properties, resourceProvider
    | order by dnsEndpoint asc, name asc, id asc
"@.Trim().ReplaceLineEndings(" ")
$resource_endpoints = az graph query --graph-query "$query" `
    | ConvertFrom-Json `
    | Select-Object -ExpandProperty data
$resource_endpoints `
    | ConvertTo-Json -Depth 25 `
    | Set-Content -Path .\ignore\resource_endpoints.json
