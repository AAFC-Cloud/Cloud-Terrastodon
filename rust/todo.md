- non-interactive invocation for pipeline usage


- create import blocks for policy assignment principal's role assignments 
    - https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs/resources/role_assignment
    https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs/resources/management_group_policy_assignment#principal_id

    ```pwsh
    az policy definition list `
    | % { $_.policyRule.then.details.roleDefinitionIds } `
    | Sort-Object -Unique
    ```


	- search terraform provider docs using ripgrep; provider CLI documentation explorer
    - https://github.com/TeamDman/horobi-transcript-utility/blob/main/chatgpt/actions/Search%20mappings.ps1


- devops add user
    - https://web.archive.org/web/20240214091953/http://www.bryancook.net/2021/03/using-azure-cli-to-call-azure-devops.html
    - https://github.com/bryanbcook/az-devops-cli-examples/blob/master/az_devops_invoke.json
        - `az devops invoke | code -`
    1. add user to devops as stakeholder
        - `az devops user add --email-id <email here> --license-type stakeholder`
    1. list devops group entitlements
        - `az devops invoke --area MemberEntitlementManagement --resource GroupEntitlements --encoding utf-8 --output json --api-version 5.1`
        - `originId`,`displayName`
    1. add user to Entra security group group
        - `az ad group member add --group <group id> --member-id <object id>`
    1. start refresh
        - `az devops invoke --area MEMInternal --resource GroupEntitlementUserApplication --encoding utf-8 --output json --api-version 5.1-preview`
    1. wait for refresh to complete
        - `az devops invoke --area LicensingRule --resource GroupLicensingRulesApplicationStatus --api-version 5.1-preview --encoding utf-8 --output json`
    1. get summary
        - `az devops invoke --area MemberEntitlementManagement --resource UserEntitlementSummary --api-version 5.1-preview --encoding utf-8 --output json --query-parameters "select=licenses,accesslevels"`
    1. get users
        - `az devops invoke --area MemberEntitlementManagement --resource MemberEntitlements --api-version 7.1-preview --encoding utf-8 --output json --query-parameters '$orderBy=name Ascending'`
        - `az devops invoke --area MemberEntitlementManagement --resource MemberEntitlements --api-version 7.1-preview --encoding utf-8 --output json --query-parameters '$filter=name eq ''john''&$orderBy=name Ascending'`

- advertise this tool
    - GitHub release (exe, linux?)
    - https://github.com/Azure/azure-cli/issues/13064#issuecomment-2118107000
    - OpenTofu slack
    - Terraform slack
    - Azuretfcommunity slack


- IPAM solution?
    - pre-allocate IP ranges by env
    - pre-allocate IP ranges by resource group
    - detect existing vnets and subnets using the ranges
    - conflict detection on rg/env mismatch
    - persist in json file
    - CLI for creating or modifying assignments