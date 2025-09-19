# TODO

## General

- PickerTui highlight matching chars like fzf

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

- Fix "Direct" PIM assignment failing to deserialize
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
