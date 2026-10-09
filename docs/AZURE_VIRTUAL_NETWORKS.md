# Azure virtual networks and subnets

`ct az vnet` is an alias for `ct azure virtual-network`. These commands use the
existing cached virtual-network inventory rather than making a separate request
for each network or subnet:

```pwsh
ct az vnet list
ct az vnet show example-vnet
ct az vnet subnet list
ct az vnet subnet list --vnet example-vnet
ct az vnet subnet show example-subnet --vnet example-vnet
```

All four commands accept `--tenant <ID|ALIAS>`, defaulting to the selected
authentication tenant.

Both `show` commands accept a validated resource name or full Azure resource ID
as their positional argument. Names and IDs are matched without case sensitivity.
Show commands require exactly one matching resource and return a single typed
record. No match or multiple matches is an error; use a full resource ID to
select a specific resource.
Subnet commands also accept `--vnet <NAME|ID>`, restricting results to subnets of
all matching virtual networks. Without it, subnets span every virtual network
in the selected tenant. This filter may match multiple networks; `subnet show`
still requires exactly one matching subnet across those networks.

List commands return arrays of typed records, including an empty array when there
are no matches. Virtual-network listing retains networks with no subnets.
`subnet list` returns an empty array if `--vnet` matches no networks.

Use the global `--output-format text`, `json`, or `facet-pretty` (`facet`) to choose
presentation. Output defaults to Facet's pretty representation in a terminal and
JSON in a pipeline, including PowerShell pipelines.

`--no-cache` (`--skip-cache` or `--clean`) refreshes the selected tenant's entire
virtual-network inventory. Subnets come from each
network's `properties.subnets`, as shown in Microsoft's
[Resource Graph subnet example](https://learn.microsoft.com/en-us/azure/governance/resource-graph/samples/starter#list-all-azure-virtual-network-subnets).
Results reflect accessible resources indexed by
[Azure Resource Graph](https://learn.microsoft.com/en-us/azure/governance/resource-graph/overview)
and can lag recent changes. These commands expose the existing network models;
they do not fetch every property from the individual ARM GET APIs. The current
virtual-network address-space model supports IPv4 prefixes.
