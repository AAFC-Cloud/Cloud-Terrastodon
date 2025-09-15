Add virtual machine image discovery
Include link to helper resource
https://az-vm-image.info/?cmd=--all+--publisher+center-for-internet-security-inc
0. Observe az cli
    - `az vm image list --all --publisher center-for-internet-security-inc --debug`
1. Fetch publishers
    - `az rest --method get --url "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/canadacentral/publishers?api-version=2024-11-01"`
2. Fetch offers
    - `az rest --method get --url "https://management.azure.com/subscriptions/{subscription_id}/providers/Microsoft.Compute/locations/canadacentral/publishers/center-for-internet-security-inc/artifacttypes/vmimage/offers?api-version=2024-07-01"`

- fix "Direct" PIM assignment failing to deserialize
- Add entra role definition fetch
- Add prediction for privileged operation like creating security group

- cloud lint: assert `name.trim()==name` for all entra groups

- Enhanced discovery navigation
"My website https://blah.agr.gc.ca/ is down"
-> NSLOOKUP
-> discover application-gateway.listeners and app-services.custom_domains that match
-> app gateway backend pools gives IPs
-> IPs give private endpoint
-> private endpoint gives resource

- Nice docs like Winnow https://docs.rs/winnow/latest/winnow/_tutorial/index.html


- fix failing tests

test role_eligibility_schedules::tests::it_works ... FAILED
test role_management_policy_assignments::tests::it_works ... FAILED