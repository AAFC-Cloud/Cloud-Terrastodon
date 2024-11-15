- replace 0-second cache expiry usage with a default command inspection/history mechanism

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


- PIM deactivation
    - list active assignments (audit log?)
    - pick many, deactivate

- devops git repo last modified date aggregator
    - identify repos that are candidates for deletion/archival


- log integration
    - Entra audit log
        - [Activity reports API overview - Microsoft Graph v1.0 | Microsoft Learn](https://learn.microsoft.com/en-us/graph/api/resources/azure-ad-auditlog-overview?view=graph-rest-1.0)
        - [How to analyze activity logs with Microsoft Graph - Microsoft Entra ID | Microsoft Learn](https://learn.microsoft.com/en-us/entra/identity/monitoring-health/howto-analyze-activity-logs-with-microsoft-graph)
        - GET `/auditLogs/directoryAudits?$filter=(activityDateTime ge 2024-07-06T18:29:40.171Z and activityDateTime le 2024-08-06T18:29:40.171Z and (startswith(initiatedBy/user/id, 'admin.john.doe') or startswith(initiatedBy/user/displayName, 'admin.john.doe') or startswith(initiatedBy/user/userPrincipalName, 'admin.john.doe') or startswith(initiatedBy/app/appId, 'admin.john.doe') or startswith(initiatedBy/app/displayName, 'admin.john.doe')))&$top=50&$orderby=activityDateTime desc`
    - AzureRM activity log
        - [az monitor activity-log | Microsoft Learn](https://learn.microsoft.com/en-us/cli/azure/monitor/activity-log?view=azure-cli-latest#az-monitor-activity-log-list)
    - What changes have recently happened to the (security group, resource group, etc) I'm currently looking at
    - What changes have recently been made by the (user, security group, service principal, etc) I'm currently looking at
	


- resource movement helper
    1. get tf resources | fzf --multi | az resource move
    2. write import blocks for new destinations
    3. tf apply



# Traversal

- Azure DevOps repos
- Resource
    - Activity logs
    - PIM assignments
    - Deployments
    - Role assignment
        - Creator, modifier
    - Parents
    - Children
- Group
    - Audit logs
    - Role assignments (entra, azurerm)
- Service Principal
    - Owners
    - Application registration
    - DevOps service connections
- Application registration
    - Custom role assignments
    - API permissions
    - Owners
- DevOps service connections
    - Projects
    - Pipelines


# IP Address

- NICs
- Public IPs
- Private IPs
- Vnets
- Subnets
- Ping
- nmap
- DNS rules

# Unused principals

Find principals with no role assignments
- no owners
- no role assignments
- no OIDC configured
- no API permissions
- expired certs

"Unknown" principal role assignments

# Cleanup

- Resource name contains "delete" or "temp"
- Action groups containing emails outside of domain