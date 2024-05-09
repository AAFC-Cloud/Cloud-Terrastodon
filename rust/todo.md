- separate `pick` into `pick_single` and `pick_many`
- create policy lookup-by-id cache to avoid duplicate fetching in `process generated` action, only lookup if cache miss
- create import blocks for policy assignment principal's role assignments 
    - https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs/resources/role_assignment
    https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs/resources/management_group_policy_assignment#principal_id

    ```pwsh
    az policy definition list `
    | % { $_.policyRule.then.details.roleDefinitionIds } `
    | Sort-Object -Unique
    ```