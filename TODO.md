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