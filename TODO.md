# TODO

## Cloud lint

- App registrations with expired (client secrets|certificates)
- App registrations with (client secrets|certificates) that are expiring in 30 days when no (client secrets|certificates) exists that is expiring in more than 30 days from now
- Users who are Owner on App Registration without having Directory Reader role or similar

## VM sku permission

- list skus
- find assignments for "Allowed virtual machine size SKUs" policy - /providers/microsoft.authorization/policydefinitions/cccc23c7-8427-4f53-ad12-b6a63eb452b3 to filter
https://stackoverflow.com/a/73192111/11141271
az vm list-sizes --location canadacentral
az rest --method get `
  --uri "https://prices.azure.com/api/retail/prices?$filter=serviceName eq 'Virtual Machines' and armRegionName eq 'canadacentral' and (contains(armSkuName, 'Promo') eq false and contains(armSkuName, 'Standard_B') eq false)"

## Oauth2

Add oauth2 permission scope browser

## Graph

Add general-purpose ms graph postman/rest api helper

## General

- PickerTui highlight matching chars like fzf
- Graceful exit when user hits esc to quit from main or submenus
- browse resource groups should look more like az role assignment list dialog
    - output full expanded form id for copy pasting as terraform import
- Update user fetching to use graph batch; add `otherMails` property
- Implement `audit_azure.rs` to use `otherMails` property

## New Types

- Add `VirtualMachineName` type
- Add `VirtualMachineSizeSku` type
    - If struct, add `fetch_all_virtual_machine_size_skus()`
    - If enum, add `VARIANTS` array
    

## Virtual Machine Images

- Add virtual machine image discovery
    - Include link to helper resource
    - https://az-vm-image.info/?cmd=--all+--publisher+center-for-internet-security-inc


0. Observe az cli
    - `az vm image list --all --publisher center-for-internet-security-inc --debug`
1. Fetch publishers
    - `az rest --method get --url "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/canadacentral/publishers?api-version=2024-11-01"`
2. Fetch offers
    - `az rest --method get --url "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/canadacentral/publishers/center-for-internet-security-inc/artifacttypes/vmimage/offers?api-version=2024-07-01"`
3. Check `accepted` in terms
    - `az vm image terms show --urn center-for-internet-security-inc:cis-windows-server:cis-windows-server2016-l2-gen2:latest`

## Privileges

- Add entra role definition fetch
- Add prediction for privileged operation like creating security group

## Linting

- cloud lint: assert `name.trim()==name` for all entra groups

## Discovery and Navigation Properties

"My website https://blah.agr.gc.ca/ is down"
-> NSLOOKUP
-> discover application-gateway.listeners and app-services.custom_domains that match
-> app gateway backend pools gives IPs
-> IPs give private endpoint
-> private endpoint gives resource

## Documentation

- Nice docs like Winnow https://docs.rs/winnow/latest/winnow/_tutorial/index.html

## Authentication

- jwt auth verification
- https://github.com/ramosbugs/openidconnect-rs
- https://www.whatismytenantid.com/
    - https://login.microsoftonline.com/agr.gc.ca/.well-known/openid-configuration
- https://login.microsoftonline.com/9da98bb1-1857-4cc3-8751-9a49e35d24cd/discovery/v2.0/keys
- https://sts.windows.net/9da98bb1-1857-4cc3-8751-9a49e35d24cd/.well-known/openid-configuration
- https://login.windows.net/common/discovery/keys
