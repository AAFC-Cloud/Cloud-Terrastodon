- create policy lookup-by-id cache to avoid duplicate fetching in `process generated` action, only lookup if cache miss
- create import blocks for policy assignment principal's role assignments 
    - https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs/resources/role_assignment
    https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs/resources/management_group_policy_assignment#principal_id

    ```pwsh
    az policy definition list `
    | % { $_.policyRule.then.details.roleDefinitionIds } `
    | Sort-Object -Unique
    ```
- create import blocks from existing terraform project
    - "I just created this flat but now I want to convert it to modules"
        - the "id" for importing a resource isn't always trivial, specific to the resource type
        - tf state pull || can generate the "to" field from this
    - Create import blocks for project to enable easy recovery if state file dies
        - just write everything to imports.tf
        - drift detection - import blocks aren't checked by terraform after initial import
            - probably want to just update the id attribute in the import block
- search terraform provider docs using ripgrep; provider CLI documentation explorer
    - https://github.com/TeamDman/horobi-transcript-utility/blob/main/chatgpt/actions/Search%20mappings.ps1